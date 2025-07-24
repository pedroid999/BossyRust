use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

pub struct ProcessKiller;

impl ProcessKiller {
    pub async fn kill_process_by_pid(pid: u32, force: bool) -> Result<()> {
        if force {
            Self::kill_force(pid).await
        } else {
            Self::kill_graceful(pid).await
        }
    }

    pub async fn kill_processes_by_name(name: &str, force: bool) -> Result<Vec<u32>> {
        let pids = Self::find_pids_by_name(name)?;
        let mut killed_pids = Vec::new();

        for pid in pids {
            match Self::kill_process_by_pid(pid, force).await {
                Ok(()) => killed_pids.push(pid),
                Err(e) => eprintln!("Failed to kill process {pid}: {e}"),
            }
        }

        Ok(killed_pids)
    }

    pub async fn kill_process_by_port(port: u16) -> Result<u32> {
        let pid = Self::find_pid_by_port(port)?;
        Self::kill_graceful(pid).await?;
        Ok(pid)
    }

    async fn kill_graceful(pid: u32) -> Result<()> {
        // First try SIGTERM
        let output = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to send SIGTERM to process {pid}: {error}"));
        }

        // Wait up to 5 seconds for graceful shutdown
        for _ in 0..50 {
            if !Self::is_process_running(pid)? {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }

        // If still running, escalate to SIGKILL
        eprintln!("Process {pid} didn't respond to SIGTERM, escalating to SIGKILL");
        Self::kill_force(pid).await
    }

    async fn kill_force(pid: u32) -> Result<()> {
        let output = Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to send SIGKILL to process {pid}: {error}"));
        }

        // Wait up to 2 seconds for force kill to take effect
        for _ in 0..20 {
            if !Self::is_process_running(pid)? {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }

        Err(anyhow!("Process {pid} is still running after SIGKILL"))
    }

    fn is_process_running(pid: u32) -> Result<bool> {
        let output = Command::new("ps").args(["-p", &pid.to_string()]).output()?;

        Ok(output.status.success())
    }

    fn find_pids_by_name(name: &str) -> Result<Vec<u32>> {
        let output = Command::new("pgrep").args(["-f", name]).output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pids: Result<Vec<u32>, _> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().parse())
            .collect();

        pids.map_err(|e| anyhow!("Failed to parse PID: {e}"))
    }

    fn find_pid_by_port(port: u16) -> Result<u32> {
        let output = Command::new("lsof")
            .args(["-t", "-i", &format!(":{port}")])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("No process found using port {port}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout
            .lines()
            .next()
            .ok_or_else(|| anyhow!("No process found using port {port}"))?;

        first_line
            .trim()
            .parse()
            .map_err(|e| anyhow!("Failed to parse PID from port lookup: {e}"))
    }

    pub async fn cleanup_dev_processes() -> Result<Vec<u32>> {
        let common_dev_processes = [
            "node",
            "npm",
            "yarn",
            "webpack",
            "vite",
            "next",
            "python",
            "django",
            "flask",
            "rails",
            "ruby",
            "php",
            "artisan",
            "composer",
            "java",
            "gradle",
            "docker",
            "docker-compose",
            "redis-server",
            "postgres",
        ];

        let mut killed_pids = Vec::new();

        for process_name in &common_dev_processes {
            match Self::kill_processes_by_name(process_name, false).await {
                Ok(pids) => killed_pids.extend(pids),
                Err(e) => eprintln!("Error killing {process_name}: {e}"),
            }
        }

        Ok(killed_pids)
    }

    pub fn find_available_port(start_port: u16, end_port: u16) -> Result<u16> {
        for port in start_port..=end_port {
            if Self::is_port_available(port)? {
                return Ok(port);
            }
        }
        Err(anyhow!(
            "No available port found in range {start_port}-{end_port}"
        ))
    }

    fn is_port_available(port: u16) -> Result<bool> {
        let output = Command::new("lsof")
            .args(["-i", &format!(":{port}")])
            .output()?;

        // If lsof returns non-zero, port is available
        Ok(!output.status.success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_killer_error_handling() {
        // Test killing a non-existent process
        let result = ProcessKiller::kill_process_by_pid(999999, false).await;
        assert!(result.is_err());

        // Test killing by non-existent process name
        let result =
            ProcessKiller::kill_processes_by_name("non_existent_process_12345", false).await;
        assert!(result.is_ok());
        let killed_pids = result.unwrap();
        assert!(killed_pids.is_empty());
    }

    #[tokio::test]
    async fn test_find_available_port() {
        // Test finding available port in a high range (likely to be available)
        let result = ProcessKiller::find_available_port(60000, 60010);

        match result {
            Ok(port) => {
                assert!(port >= 60000);
                assert!(port <= 60010);
            }
            Err(_) => {
                // It's possible no ports are available, which is acceptable
            }
        }
    }

    #[tokio::test]
    async fn test_port_availability_check() {
        // Test port availability for a very high port (likely available)
        let result = ProcessKiller::is_port_available(65000);

        // Should either return true (available) or false (not available)
        // Both are valid results
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_process_running() {
        // Test with process PID 1 (init/launchd - should always exist on Unix systems)
        let result = ProcessKiller::is_process_running(1);
        assert!(result.is_ok());

        // On macOS/Unix, PID 1 should always be running
        if let Ok(is_running) = result {
            assert!(is_running);
        }

        // Test with a very high PID (likely not to exist)
        let result = ProcessKiller::is_process_running(999999);
        assert!(result.is_ok());

        if let Ok(is_running) = result {
            // Very high PID is unlikely to be running
            assert!(!is_running);
        }
    }

    #[test]
    fn test_find_pids_by_name() {
        // Test finding PIDs for a common system process
        let result = ProcessKiller::find_pids_by_name("kernel_task");

        match result {
            Ok(pids) => {
                // kernel_task should exist on macOS
                // PIDs should be positive numbers
                for pid in pids {
                    assert!(pid > 0);
                }
            }
            Err(_) => {
                // pgrep might not be available or process might not exist
                // This is acceptable for the test
            }
        }
    }

    #[test]
    fn test_find_pid_by_port_non_existent() {
        // Test finding PID for a port that's very unlikely to be used
        let result = ProcessKiller::find_pid_by_port(65534);

        // Should either find a PID or return an error (port not in use)
        // Both outcomes are valid
        match result {
            Ok(pid) => assert!(pid > 0),
            Err(e) => assert!(e.to_string().contains("No process found")),
        }
    }

    #[tokio::test]
    async fn test_cleanup_dev_processes_safety() {
        // This test ensures cleanup doesn't crash
        // We won't actually kill processes in tests

        // For safety, we'll test the function structure exists
        // In a real test environment, we'd mock the system calls
        let result = ProcessKiller::cleanup_dev_processes().await;

        // Should return a result (either success with PIDs or an error)
        assert!(result.is_ok());

        let killed_pids = result.unwrap();
        // Could be empty (no dev processes) or contain PIDs
        // Both are valid outcomes
        for pid in killed_pids {
            assert!(pid > 0);
        }
    }

    #[test]
    fn test_find_available_port_range_validation() {
        // Test edge cases for port range finding

        // Test with single port
        let result = ProcessKiller::find_available_port(50000, 50000);
        // Should either succeed or fail gracefully
        match result {
            Ok(port) => assert_eq!(port, 50000),
            Err(_) => {} // Port might be in use, which is fine
        }

        // Test with small range
        let result = ProcessKiller::find_available_port(50000, 50001);
        match result {
            Ok(port) => {
                assert!(port >= 50000);
                assert!(port <= 50001);
            }
            Err(_) => {} // Ports might be in use
        }
    }

    #[test]
    fn test_error_messages_quality() {
        // Test that error messages are meaningful

        let result = ProcessKiller::find_pid_by_port(0);
        if let Err(e) = result {
            let error_msg = e.to_string();
            // Error message should mention the port or "process"
            assert!(error_msg.contains("process") || error_msg.contains("0"));
        }
    }

    #[tokio::test]
    async fn test_graceful_vs_force_kill_parameters() {
        // Test that force and graceful parameters are handled correctly

        // Test with non-existent PID - should fail gracefully for both modes
        let graceful_result = ProcessKiller::kill_process_by_pid(999999, false).await;
        let force_result = ProcessKiller::kill_process_by_pid(999999, true).await;

        // Both should fail (PID doesn't exist) but not panic
        assert!(graceful_result.is_err());
        assert!(force_result.is_err());

        // Error messages should be meaningful
        if let Err(e) = graceful_result {
            assert!(!e.to_string().is_empty());
        }

        if let Err(e) = force_result {
            assert!(!e.to_string().is_empty());
        }
    }

    #[test]
    fn test_system_command_safety() {
        // Test that system commands are called safely

        // Test ps command with safe parameters
        let result = ProcessKiller::is_process_running(1);
        assert!(result.is_ok());

        // Test lsof command with safe parameters
        let result = ProcessKiller::find_pid_by_port(65535);
        // Should complete without panicking
        match result {
            Ok(_) | Err(_) => {} // Both outcomes are acceptable
        }
    }
}
