use crate::network::{NetworkUtils, PortManager};
use crate::process::{ProcessKiller, ProcessMonitor};
use anyhow::Result;

pub struct CliHandler;

impl CliHandler {
    pub async fn show_port_info(port: u16) -> Result<()> {
        let ports = PortManager::get_port_by_number(port)?;

        if ports.is_empty() {
            println!("No processes found using port {port}");
            return Ok(());
        }

        println!("Port {port} information:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for port_info in ports {
            println!("Protocol: {:?}", port_info.protocol);
            println!("State: {:?}", port_info.state);
            println!("Local Address: {}", port_info.local_address);

            if let Some(remote) = &port_info.remote_address {
                println!("Remote Address: {remote}");
            }

            if let Some(pid) = port_info.pid {
                println!("PID: {pid}");
            }

            if let Some(ref process_name) = port_info.process_name {
                println!("Process: {process_name}");
            }

            if let Some(service) = port_info.get_service_suggestion() {
                println!("Service: {service}");
            }

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        Ok(())
    }

    pub async fn kill_port(port: u16) -> Result<()> {
        println!("Killing process using port {port}...");

        match ProcessKiller::kill_process_by_port(port).await {
            Ok(pid) => {
                println!("✅ Successfully killed process {pid} using port {port}");
            }
            Err(e) => {
                eprintln!("❌ Failed to kill process on port {port}: {e}");
                std::process::exit(1);
            }
        }

        Ok(())
    }

    pub async fn show_ports(common: bool, listening: bool) -> Result<()> {
        let ports = if listening {
            PortManager::get_listening_ports()?
        } else if common {
            PortManager::get_development_ports()?
        } else {
            PortManager::get_all_ports()?
        };

        if ports.is_empty() {
            println!("No ports found");
            return Ok(());
        }

        println!("Ports ({}):", ports.len());
        println!("┌─────────┬─────────┬────────────┬─────────┬─────────────────────┬──────────────────────┐");
        println!("│  Port   │ Proto   │   State    │   PID   │       Process       │       Service        │");
        println!("├─────────┼─────────┼────────────┼─────────┼─────────────────────┼──────────────────────┤");

        for port in ports {
            let protocol = format!("{:?}", port.protocol);
            let state = format!("{:?}", port.state);
            let pid = port.pid.map_or("-".to_string(), |p| p.to_string());
            let process = port.process_name.as_deref().unwrap_or("-");
            let service = port.get_service_suggestion().unwrap_or("-".to_string());

            println!(
                "│ {:>7} │ {:>7} │ {:>10} │ {:>7} │ {:>19} │ {:>20} │",
                port.port,
                &protocol[..std::cmp::min(protocol.len(), 7)],
                &state[..std::cmp::min(state.len(), 10)],
                &pid[..std::cmp::min(pid.len(), 7)],
                &process[..std::cmp::min(process.len(), 19)],
                &service[..std::cmp::min(service.len(), 20)]
            );
        }

        println!("└─────────┴─────────┴────────────┴─────────┴─────────────────────┴──────────────────────┘");

        Ok(())
    }

    pub async fn kill_process(name: &str, force: bool) -> Result<()> {
        println!("Killing processes matching '{name}'...");

        match ProcessKiller::kill_processes_by_name(name, force).await {
            Ok(pids) => {
                if pids.is_empty() {
                    println!("No processes found matching '{name}'");
                } else {
                    println!(
                        "✅ Successfully killed {} process(es): {pids:?}",
                        pids.len()
                    );
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to kill processes: {e}");
                std::process::exit(1);
            }
        }

        Ok(())
    }

    pub async fn show_processes(top_cpu: bool, top_memory: bool, limit: usize) -> Result<()> {
        let mut monitor = ProcessMonitor::new();

        let processes = if top_cpu {
            monitor.get_top_cpu_processes(limit)
        } else if top_memory {
            monitor.get_top_memory_processes(limit)
        } else {
            let mut procs = monitor.get_processes();
            procs.truncate(limit);
            procs
        };

        if processes.is_empty() {
            println!("No processes found");
            return Ok(());
        }

        let title = if top_cpu {
            format!("Top {limit} CPU Consumers")
        } else if top_memory {
            format!("Top {limit} Memory Consumers")
        } else {
            format!("Processes (showing {limit})")
        };

        println!("{title}:");
        println!("┌──────────┬─────────────────────┬─────────┬─────────────┬──────────────┐");
        println!("│   PID    │       Process       │  CPU %  │   Memory    │    Status    │");
        println!("├──────────┼─────────────────────┼─────────┼─────────────┼──────────────┤");

        for process in processes {
            let name = if process.name.len() > 19 {
                format!("{}…", &process.name[..18])
            } else {
                process.name.clone()
            };

            let status = if process.status.len() > 12 {
                format!("{}…", &process.status[..11])
            } else {
                process.status.clone()
            };

            println!(
                "│ {:>8} │ {:>19} │ {:>7.1} │ {:>11} │ {:>12} │",
                process.pid,
                name,
                process.cpu_usage,
                process.format_memory(),
                status
            );
        }

        println!("└──────────┴─────────────────────┴─────────┴─────────────┴──────────────┘");

        Ok(())
    }

    pub async fn cleanup_processes(dev: bool) -> Result<()> {
        if dev {
            println!("Cleaning up development processes...");

            match ProcessKiller::cleanup_dev_processes().await {
                Ok(pids) => {
                    if pids.is_empty() {
                        println!("No development processes found to cleanup");
                    } else {
                        println!(
                            "✅ Cleaned up {} development processes: {pids:?}",
                            pids.len()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to cleanup processes: {e}");
                    std::process::exit(1);
                }
            }
        } else {
            println!("Please specify --dev to cleanup development processes");
        }

        Ok(())
    }

    pub async fn find_available_port(start: u16, end: u16) -> Result<()> {
        println!("Searching for available ports in range {start}-{end}...");

        match ProcessKiller::find_available_port(start, end) {
            Ok(port) => {
                println!("✅ Available port found: {port}");

                // Show suggestions for common development ports
                if NetworkUtils::is_development_port(port) {
                    if let Some(service) = NetworkUtils::get_well_known_ports().get(&port) {
                        println!("💡 This port is commonly used for: {service}");
                    }
                }
            }
            Err(e) => {
                println!("❌ {e}");

                // Suggest alternatives
                let alternatives = NetworkUtils::suggest_alternative_port(start);
                if !alternatives.is_empty() {
                    println!("💡 Consider trying these alternative ports: {alternatives:?}");
                }

                std::process::exit(1);
            }
        }

        Ok(())
    }
}
