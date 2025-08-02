#[cfg(test)]
use mockall::predicate::*;
#[cfg(test)]
use mockall::mock;
use std::process::Output;
use std::os::unix::process::ExitStatusExt;
use anyhow::Result;

/// Mock trait for system commands to enable testable system interactions
pub trait SystemCommandExecutor {
    fn execute_command(&self, command: &str, args: &[String]) -> Result<Output>;
    fn is_process_running(&self, pid: u32) -> Result<bool>;
    fn get_processes(&self) -> Result<String>;
    fn get_port_info(&self) -> Result<String>;
    fn get_network_connections(&self) -> Result<String>;
}

#[cfg(test)]
mock! {
    pub SystemCommand {}
    
    impl SystemCommandExecutor for SystemCommand {
        fn execute_command(&self, command: &str, args: &[String]) -> Result<Output>;
        fn is_process_running(&self, pid: u32) -> Result<bool>;
        fn get_processes(&self) -> Result<String>;
        fn get_port_info(&self) -> Result<String>;
        fn get_network_connections(&self) -> Result<String>;
    }
}

/// Mock system command outputs for testing
pub struct MockSystemOutputs;

impl MockSystemOutputs {
    /// Mock ps command output with realistic process data
    pub fn mock_ps_output() -> String {
        r#"  PID  %CPU %MEM COMMAND
    1   0.0  0.1 /sbin/launchd
  100  45.2  2.1 node server.js
  101  23.1  1.8 python manage.py runserver
  102  15.8  8.3 /Applications/Google Chrome.app/Contents/MacOS/Google Chrome
  103   8.4  4.2 /Applications/Visual Studio Code.app/Contents/MacOS/Electron
  104  12.1  3.1 docker-proxy -proto tcp -host-ip 0.0.0.0
  105   3.2  1.5 rust-analyzer"#.to_string()
    }

    /// Mock lsof command output for port information
    pub fn mock_lsof_output() -> String {
        r#"COMMAND   PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node      100 user    12u  IPv4 0x1234567890      0t0  TCP *:3000 (LISTEN)
python    101 user    13u  IPv4 0x2345678901      0t0  TCP *:8000 (LISTEN)
docker    104 user    14u  IPv4 0x3456789012      0t0  TCP *:8080 (LISTEN)
postgres  106 user    15u  IPv4 0x4567890123      0t0  TCP *:5432 (LISTEN)"#.to_string()
    }

    /// Mock netstat command output for network connections
    pub fn mock_netstat_output() -> String {
        r#"Active Internet connections
Proto Recv-Q Send-Q  Local Address          Foreign Address        (state)    
tcp4       0      0  127.0.0.1.3000         *.*                    LISTEN     
tcp4       0      0  127.0.0.1.8000         *.*                    LISTEN     
tcp4       0      0  127.0.0.1.8080         *.*                    LISTEN     
tcp4       0      0  127.0.0.1.5432         *.*                    LISTEN     
tcp4       0      0  127.0.0.1.1234         192.168.1.1.80         ESTABLISHED
tcp4       0      0  127.0.0.1.5678         1.1.1.1.443           ESTABLISHED"#.to_string()
    }

    /// Mock kill command success
    pub fn mock_kill_success() -> Output {
        Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        }
    }

    /// Mock kill command failure
    pub fn mock_kill_failure() -> Output {
        Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: b"kill: No such process".to_vec(),
        }
    }

    /// Mock pgrep output for finding processes by name
    pub fn mock_pgrep_output(pids: &[u32]) -> String {
        pids.iter().map(|pid| pid.to_string()).collect::<Vec<_>>().join("\n")
    }
}

/// Helper to create a mock system command executor for testing
#[cfg(test)]
pub fn create_mock_system_executor() -> MockSystemCommand {
    let mut mock = MockSystemCommand::new();
    
    // Setup default expectations
    mock.expect_get_processes()
        .returning(|| Ok(MockSystemOutputs::mock_ps_output()));
        
    mock.expect_get_port_info()
        .returning(|| Ok(MockSystemOutputs::mock_lsof_output()));
        
    mock.expect_get_network_connections()
        .returning(|| Ok(MockSystemOutputs::mock_netstat_output()));
    
    mock
}

/// Mock terminal backend for TUI testing
pub mod tui_mocks {
    use ratatui::backend::Backend;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use std::io::{self, Write};

    /// Mock backend that captures render operations for testing
    pub struct MockBackend {
        pub size: Rect,
        pub buffer: Buffer,
        pub draw_calls: Vec<Buffer>,
    }

    impl MockBackend {
        pub fn new(width: u16, height: u16) -> Self {
            let size = Rect::new(0, 0, width, height);
            Self {
                size,
                buffer: Buffer::empty(size),
                draw_calls: Vec::new(),
            }
        }

        /// Get the number of draw calls made
        pub fn draw_call_count(&self) -> usize {
            self.draw_calls.len()
        }

        /// Get the last rendered buffer
        pub fn last_buffer(&self) -> Option<&Buffer> {
            self.draw_calls.last()
        }

        /// Check if a string appears in the last rendered buffer
        pub fn contains_text(&self, text: &str) -> bool {
            if let Some(buffer) = self.last_buffer() {
                buffer.content().iter().any(|cell| cell.symbol().contains(text))
            } else {
                false
            }
        }
    }

    impl Backend for MockBackend {
        fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
        where
            I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
        {
            for (x, y, cell) in content {
                self.buffer.get_mut(x, y).clone_from(cell);
            }
            self.draw_calls.push(self.buffer.clone());
            Ok(())
        }

        fn hide_cursor(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn show_cursor(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
            Ok((0, 0))
        }

        fn set_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
            Ok(())
        }

        fn clear(&mut self) -> io::Result<()> {
            self.buffer.reset();
            Ok(())
        }

        fn size(&self) -> io::Result<Rect> {
            Ok(self.size)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
            Ok(ratatui::backend::WindowSize {
                columns_rows: (self.size.width, self.size.height).into(),
                pixels: (0, 0).into(),
            })
        }
    }

    impl Write for MockBackend {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_ps_output_parsing() {
        let output = MockSystemOutputs::mock_ps_output();
        assert!(output.contains("node server.js"));
        assert!(output.contains("45.2"));
        assert!(output.contains("python manage.py"));
    }

    #[test]
    fn test_mock_lsof_output_parsing() {
        let output = MockSystemOutputs::mock_lsof_output();
        assert!(output.contains(":3000"));
        assert!(output.contains("LISTEN"));
        assert!(output.contains("node"));
    }

    #[test]
    fn test_mock_netstat_output_parsing() {
        let output = MockSystemOutputs::mock_netstat_output();
        assert!(output.contains("127.0.0.1.3000"));
        assert!(output.contains("ESTABLISHED"));
        assert!(output.contains("LISTEN"));
    }

    #[test]
    fn test_create_mock_system_executor() {
        let _mock = create_mock_system_executor();
        // Test that the mock can be created without panicking
        assert!(true);
    }
}