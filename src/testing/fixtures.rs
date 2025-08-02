use crate::network::{ConnectionInfo, PortInfo, Protocol, ConnectionState};
use crate::process::ProcessInfo;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Create test fixture for ProcessInfo
pub fn create_test_process(pid: u32, name: &str, cpu: f32, memory: u64) -> ProcessInfo {
    ProcessInfo {
        pid,
        name: name.to_string(),
        cpu_usage: cpu,
        memory,
        parent_pid: if pid > 1 { Some(1) } else { None },
        status: "Running".to_string(),
        start_time: 1000,
        user_id: Some(501),
        executable_path: Some(format!("/usr/bin/{}", name)),
        command_line: vec![name.to_string()],
    }
}

/// Create test fixture for PortInfo
pub fn create_test_port(port: u16, protocol: Protocol, pid: Option<u32>) -> PortInfo {
    PortInfo {
        port,
        protocol,
        pid,
        process_name: pid.map(|p| format!("process_{}", p)),
        local_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
        remote_address: None,
        state: ConnectionState::Listen,
        service_name: None,
    }
}

/// Create test fixture for ConnectionInfo
pub fn create_test_connection(
    local_port: u16,
    remote_port: u16,
    pid: Option<u32>,
) -> ConnectionInfo {
    ConnectionInfo {
        protocol: Protocol::Tcp,
        local_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), local_port),
        remote_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), remote_port),
        pid,
        process_name: pid.map(|p| format!("process_{}", p)),
    }
}

/// Create a set of realistic test processes for integration testing
pub fn create_realistic_test_processes() -> Vec<ProcessInfo> {
    vec![
        create_test_process(1, "kernel_task", 5.0, 1024 * 1024 * 1024), // 1GB system process
        create_test_process(100, "node", 45.2, 512 * 1024 * 1024),      // 512MB Node.js
        create_test_process(101, "python", 23.1, 256 * 1024 * 1024),    // 256MB Python
        create_test_process(102, "chrome", 15.8, 2 * 1024 * 1024 * 1024), // 2GB Chrome
        create_test_process(103, "code", 8.4, 400 * 1024 * 1024),       // 400MB VS Code
        create_test_process(104, "docker", 12.1, 300 * 1024 * 1024),    // 300MB Docker
        create_test_process(105, "rust-analyzer", 3.2, 150 * 1024 * 1024), // 150MB rust-analyzer
    ]
}

/// Create a set of realistic test ports for integration testing
pub fn create_realistic_test_ports() -> Vec<PortInfo> {
    vec![
        create_test_port(3000, Protocol::Tcp, Some(100)), // Node.js dev server
        create_test_port(8080, Protocol::Tcp, Some(104)), // Docker container
        create_test_port(5432, Protocol::Tcp, Some(106)), // PostgreSQL
        create_test_port(6379, Protocol::Tcp, Some(107)), // Redis
        create_test_port(3306, Protocol::Tcp, Some(108)), // MySQL
        create_test_port(53, Protocol::Udp, Some(1)),     // DNS
        create_test_port(22, Protocol::Tcp, Some(1)),     // SSH
    ]
}

/// Create test connections for integration testing
pub fn create_realistic_test_connections() -> Vec<ConnectionInfo> {
    vec![
        create_test_connection(3000, 80, Some(100)),   // Node.js to HTTP
        create_test_connection(3001, 443, Some(100)),  // Node.js to HTTPS
        create_test_connection(8080, 443, Some(104)),  // Docker to HTTPS
        create_test_connection(1234, 5432, Some(102)), // Chrome to DB
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_process() {
        let process = create_test_process(123, "test", 50.0, 1024);
        assert_eq!(process.pid, 123);
        assert_eq!(process.name, "test");
        assert_eq!(process.cpu_usage, 50.0);
        assert_eq!(process.memory, 1024);
        assert_eq!(process.status, "Running");
    }

    #[test]
    fn test_create_test_port() {
        let port = create_test_port(8080, Protocol::Tcp, Some(123));
        assert_eq!(port.port, 8080);
        assert_eq!(port.protocol, Protocol::Tcp);
        assert_eq!(port.pid, Some(123));
        assert_eq!(port.state, ConnectionState::Listen);
    }

    #[test]
    fn test_realistic_fixtures_sizes() {
        let processes = create_realistic_test_processes();
        let ports = create_realistic_test_ports();
        let connections = create_realistic_test_connections();

        assert_eq!(processes.len(), 7);
        assert_eq!(ports.len(), 7);
        assert_eq!(connections.len(), 4);
    }

    #[test]
    fn test_realistic_processes_have_variety() {
        let processes = create_realistic_test_processes();
        
        // Check we have system processes
        assert!(processes.iter().any(|p| p.name == "kernel_task"));
        
        // Check we have development processes
        assert!(processes.iter().any(|p| p.name == "node"));
        assert!(processes.iter().any(|p| p.name == "python"));
        
        // Check CPU usage varies
        let cpu_values: Vec<f32> = processes.iter().map(|p| p.cpu_usage).collect();
        assert!(cpu_values.iter().any(|&cpu| cpu > 40.0));
        assert!(cpu_values.iter().any(|&cpu| cpu < 10.0));
    }
}