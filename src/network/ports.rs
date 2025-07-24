use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: Protocol,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub local_address: SocketAddr,
    pub remote_address: Option<SocketAddr>,
    pub state: ConnectionState,
    pub service_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Listen,
    Established,
    TimeWait,
    CloseWait,
    FinWait1,
    FinWait2,
    SynSent,
    SynReceived,
    Closed,
    Unknown,
}

impl From<&str> for ConnectionState {
    fn from(state: &str) -> Self {
        match state.to_uppercase().as_str() {
            "LISTEN" => ConnectionState::Listen,
            "ESTABLISHED" => ConnectionState::Established,
            "TIME_WAIT" => ConnectionState::TimeWait,
            "CLOSE_WAIT" => ConnectionState::CloseWait,
            "FIN_WAIT1" => ConnectionState::FinWait1,
            "FIN_WAIT2" => ConnectionState::FinWait2,
            "SYN_SENT" => ConnectionState::SynSent,
            "SYN_RCVD" => ConnectionState::SynReceived,
            "CLOSED" => ConnectionState::Closed,
            _ => ConnectionState::Unknown,
        }
    }
}

impl PortInfo {
    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();

        // Handle port search patterns
        if let Some(port_query) = query.strip_prefix(':') {
            // Range search: :5432-5434
            if port_query.contains('-') {
                if let Some((start, end)) = port_query.split_once('-') {
                    if let (Ok(start_port), Ok(end_port)) =
                        (start.parse::<u16>(), end.parse::<u16>())
                    {
                        return self.port >= start_port && self.port <= end_port;
                    }
                }
            } else {
                // Exact port search: :3000
                if let Ok(search_port) = port_query.parse::<u16>() {
                    return self.port == search_port;
                }
            }
        }

        // Process name search
        if let Some(ref process_name) = self.process_name {
            if process_name.to_lowercase().contains(&query) {
                return true;
            }
        }

        // Service name search
        if let Some(ref service_name) = self.service_name {
            if service_name.to_lowercase().contains(&query) {
                return true;
            }
        }

        // Port number search
        self.port.to_string().contains(&query)
    }

    pub fn is_development_port(&self) -> bool {
        matches!(self.port,
            3000 | 3001 | 3002 | // React, Next.js
            5000 | 5001 | 5002 | // Flask, various dev servers
            8000 | 8080 | 8888 | // Django, generic HTTP
            4200 | // Angular
            5432 | // PostgreSQL
            3306 | // MySQL
            6379 | // Redis
            27017 | // MongoDB
            9200 | 9300 | // Elasticsearch
            5672 | // RabbitMQ
            1433 | // SQL Server
            8081..=8090 | // Common dev ports
            9000..=9010   // Common dev ports
        )
    }

    pub fn get_service_suggestion(&self) -> Option<String> {
        match self.port {
            80 => Some("HTTP".to_string()),
            443 => Some("HTTPS".to_string()),
            22 => Some("SSH".to_string()),
            21 => Some("FTP".to_string()),
            25 => Some("SMTP".to_string()),
            53 => Some("DNS".to_string()),
            3000 => Some("React/Next.js Dev Server".to_string()),
            3001 => Some("Create React App".to_string()),
            4200 => Some("Angular Dev Server".to_string()),
            5000 => Some("Flask Dev Server".to_string()),
            8000 => Some("Django Dev Server".to_string()),
            8080 => Some("HTTP Alternate/Tomcat".to_string()),
            5432 => Some("PostgreSQL".to_string()),
            3306 => Some("MySQL".to_string()),
            6379 => Some("Redis".to_string()),
            27017 => Some("MongoDB".to_string()),
            9200 => Some("Elasticsearch".to_string()),
            5672 => Some("RabbitMQ".to_string()),
            _ => None,
        }
    }
}

pub struct PortManager;

impl PortManager {
    pub fn get_all_ports() -> Result<Vec<PortInfo>> {
        let mut ports = Vec::new();

        // Get TCP connections
        ports.extend(Self::get_tcp_connections()?);

        // Get UDP connections
        ports.extend(Self::get_udp_connections()?);

        Ok(ports)
    }

    pub fn get_listening_ports() -> Result<Vec<PortInfo>> {
        Ok(Self::get_all_ports()?
            .into_iter()
            .filter(|port| port.state == ConnectionState::Listen)
            .collect())
    }

    pub fn get_port_by_number(port_number: u16) -> Result<Vec<PortInfo>> {
        Ok(Self::get_all_ports()?
            .into_iter()
            .filter(|port| port.port == port_number)
            .collect())
    }

    pub fn get_development_ports() -> Result<Vec<PortInfo>> {
        Ok(Self::get_all_ports()?
            .into_iter()
            .filter(|port| port.is_development_port())
            .collect())
    }

    fn get_tcp_connections() -> Result<Vec<PortInfo>> {
        let output = Command::new("netstat")
            .args(["-an", "-p", "tcp"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to run netstat for TCP connections"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_netstat_output(&stdout, Protocol::Tcp)
    }

    fn get_udp_connections() -> Result<Vec<PortInfo>> {
        let output = Command::new("netstat")
            .args(["-an", "-p", "udp"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to run netstat for UDP connections"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_netstat_output(&stdout, Protocol::Udp)
    }

    fn parse_netstat_output(output: &str, protocol: Protocol) -> Result<Vec<PortInfo>> {
        let mut ports = Vec::new();
        let pid_map = Self::get_pid_port_mapping()?;

        for line in output.lines() {
            if let Some(port_info) = Self::parse_netstat_line(line, &protocol, &pid_map) {
                ports.push(port_info);
            }
        }

        Ok(ports)
    }

    fn parse_netstat_line(
        line: &str,
        protocol: &Protocol,
        pid_map: &HashMap<u16, (u32, String)>,
    ) -> Option<PortInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 4 {
            return None;
        }

        // Skip header lines
        if parts[0] == "Active" || parts[0] == "Proto" {
            return None;
        }

        let local_addr_str = parts.get(3)?;
        let state_str = if protocol == &Protocol::Tcp {
            parts.get(5).unwrap_or(&"UNKNOWN")
        } else {
            "LISTEN" // UDP doesn't have states in the same way
        };

        let local_addr = Self::parse_socket_addr(local_addr_str)?;
        let port = local_addr.port();

        let (pid, process_name) = pid_map
            .get(&port)
            .map(|(pid, name)| (Some(*pid), Some(name.clone())))
            .unwrap_or((None, None));

        let remote_addr = if parts.len() > 4 && parts[4] != "*.*" {
            Self::parse_socket_addr(parts[4])
        } else {
            None
        };

        Some(PortInfo {
            port,
            protocol: protocol.clone(),
            pid,
            process_name,
            local_address: local_addr,
            remote_address: remote_addr,
            state: ConnectionState::from(state_str),
            service_name: None, // We'll populate this separately if needed
        })
    }

    fn parse_socket_addr(addr_str: &str) -> Option<SocketAddr> {
        // Handle different formats: *.port, ip.port, [ipv6]:port
        if addr_str.starts_with('*') {
            // *:port or *.port format
            let port_str = addr_str.split(['.', ':']).next_back()?;
            let port: u16 = port_str.parse().ok()?;
            return Some(SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port));
        }

        // Try parsing as regular socket address
        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            return Some(addr);
        }

        // Handle IPv4 dot notation: 127.0.0.1.8080
        if let Some(last_dot) = addr_str.rfind('.') {
            let ip_part = &addr_str[..last_dot];
            let port_part = &addr_str[last_dot + 1..];

            if let (Ok(ip), Ok(port)) = (ip_part.parse::<IpAddr>(), port_part.parse::<u16>()) {
                return Some(SocketAddr::new(ip, port));
            }
        }

        None
    }

    fn get_pid_port_mapping() -> Result<HashMap<u16, (u32, String)>> {
        let output = Command::new("lsof").args(["-i", "-P", "-n"]).output()?;

        if !output.status.success() {
            return Ok(HashMap::new()); // Return empty map if lsof fails
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut mapping = HashMap::new();

        let re = Regex::new(r"(\S+)\s+(\d+)\s+\S+\s+\S+\s+\S+\s+\S+\s+\S+\s+.*?:(\d+)")?;

        for line in stdout.lines() {
            if let Some(captures) = re.captures(line) {
                if let (Some(process_name), Some(pid_str), Some(port_str)) =
                    (captures.get(1), captures.get(2), captures.get(3))
                {
                    if let (Ok(pid), Ok(port)) = (
                        pid_str.as_str().parse::<u32>(),
                        port_str.as_str().parse::<u16>(),
                    ) {
                        mapping.insert(port, (pid, process_name.as_str().to_string()));
                    }
                }
            }
        }

        Ok(mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::NetworkUtils;
    use std::net::{IpAddr, Ipv4Addr};

    fn create_test_port_info() -> PortInfo {
        PortInfo {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: Some(1234),
            process_name: Some("node".to_string()),
            local_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
            remote_address: None,
            state: ConnectionState::Listen,
            service_name: None,
        }
    }

    #[test]
    fn test_port_info_creation() {
        let port_info = create_test_port_info();
        assert_eq!(port_info.port, 3000);
        assert_eq!(port_info.protocol, Protocol::Tcp);
        assert_eq!(port_info.state, ConnectionState::Listen);
        assert_eq!(port_info.pid, Some(1234));
        assert_eq!(port_info.process_name, Some("node".to_string()));
    }

    #[test]
    fn test_port_search_patterns() {
        let port_info = create_test_port_info();

        // Test port number search
        assert!(port_info.matches_search(":3000"));
        assert!(!port_info.matches_search(":8080"));

        // Test process name search
        assert!(port_info.matches_search("node"));
        assert!(port_info.matches_search("NODE")); // case insensitive
        assert!(!port_info.matches_search("python"));

        // Test general port search
        assert!(port_info.matches_search("3000"));
    }

    #[test]
    fn test_port_range_search() {
        let port_info = create_test_port_info();

        // Test port range search
        assert!(port_info.matches_search(":2999-3001"));
        assert!(port_info.matches_search(":3000-3000"));
        assert!(!port_info.matches_search(":3001-3002"));
    }

    #[test]
    fn test_development_port_detection() {
        let mut port_info = create_test_port_info();

        // Test common development ports
        port_info.port = 3000;
        assert!(port_info.is_development_port());

        port_info.port = 4200;
        assert!(port_info.is_development_port());

        port_info.port = 5432;
        assert!(port_info.is_development_port());

        // Test non-development port
        port_info.port = 443;
        assert!(!port_info.is_development_port());
    }

    #[test]
    fn test_service_suggestions() {
        let mut port_info = create_test_port_info();

        // Test well-known port suggestions
        port_info.port = 80;
        assert_eq!(port_info.get_service_suggestion(), Some("HTTP".to_string()));

        port_info.port = 443;
        assert_eq!(
            port_info.get_service_suggestion(),
            Some("HTTPS".to_string())
        );

        port_info.port = 3000;
        assert_eq!(
            port_info.get_service_suggestion(),
            Some("React/Next.js Dev Server".to_string())
        );

        port_info.port = 5432;
        assert_eq!(
            port_info.get_service_suggestion(),
            Some("PostgreSQL".to_string())
        );

        // Test unknown port
        port_info.port = 12345;
        assert_eq!(port_info.get_service_suggestion(), None);
    }

    #[test]
    fn test_protocol_matching() {
        let mut port_info = create_test_port_info();

        port_info.protocol = Protocol::Tcp;
        assert_eq!(port_info.protocol, Protocol::Tcp);

        port_info.protocol = Protocol::Udp;
        assert_eq!(port_info.protocol, Protocol::Udp);

        assert_ne!(Protocol::Tcp, Protocol::Udp);
    }

    #[test]
    fn test_connection_state_parsing() {
        assert_eq!(ConnectionState::from("LISTEN"), ConnectionState::Listen);
        assert_eq!(
            ConnectionState::from("ESTABLISHED"),
            ConnectionState::Established
        );
        assert_eq!(
            ConnectionState::from("TIME_WAIT"),
            ConnectionState::TimeWait
        );
        assert_eq!(
            ConnectionState::from("UNKNOWN_STATE"),
            ConnectionState::Unknown
        );
    }

    #[test]
    fn test_socket_addr_parsing() {
        // Test IPv4 address parsing
        let addr = PortManager::parse_socket_addr("127.0.0.1.3000");
        assert!(addr.is_some());
        let addr = addr.unwrap();
        assert_eq!(addr.port(), 3000);

        // Test wildcard address parsing
        let addr = PortManager::parse_socket_addr("*.3000");
        assert!(addr.is_some());
        let addr = addr.unwrap();
        assert_eq!(addr.port(), 3000);
        assert!(addr.ip().is_unspecified());

        // Test invalid address
        let addr = PortManager::parse_socket_addr("invalid");
        assert!(addr.is_none());
    }

    #[test]
    fn test_common_ports_mapping() {
        let common_ports = NetworkUtils::get_well_known_ports();

        assert_eq!(common_ports.get(&80), Some(&"HTTP"));
        assert_eq!(common_ports.get(&443), Some(&"HTTPS"));
        assert_eq!(common_ports.get(&3000), Some(&"React/Next.js Dev"));
        assert_eq!(common_ports.get(&5432), Some(&"PostgreSQL"));
        assert_eq!(common_ports.get(&65534), None);
    }

    #[test]
    fn test_invalid_search_patterns() {
        let port_info = create_test_port_info();

        // Invalid port range format
        assert!(!port_info.matches_search(":abc-def"));
        assert!(!port_info.matches_search(":3000-"));
        assert!(!port_info.matches_search(":-3000"));

        // Empty searches should not match port patterns
        assert!(!port_info.matches_search(":"));
        // Empty search should match (falls through to name search which contains "")
        // assert!(!port_info.matches_search(""));  // This actually matches everything
    }

    #[test]
    fn test_port_manager_error_handling() {
        // These tests would ideally mock the system commands,
        // but for now we'll just test that the functions don't panic

        // Test with invalid port numbers in edge cases
        let result = PortManager::get_port_by_number(0);
        assert!(result.is_ok());

        let result = PortManager::get_port_by_number(65535);
        assert!(result.is_ok());
    }

    #[test]
    fn test_port_filtering() {
        // Create test data
        let listening_port = PortInfo {
            port: 8080,
            protocol: Protocol::Tcp,
            pid: Some(5678),
            process_name: Some("python".to_string()),
            local_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080),
            remote_address: None,
            state: ConnectionState::Listen,
            service_name: None,
        };

        let established_port = PortInfo {
            port: 443,
            protocol: Protocol::Tcp,
            pid: Some(9012),
            process_name: Some("chrome".to_string()),
            local_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 443),
            remote_address: Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(172, 217, 14, 206)),
                443,
            )),
            state: ConnectionState::Established,
            service_name: None,
        };

        // Test state filtering
        assert_eq!(listening_port.state, ConnectionState::Listen);
        assert_eq!(established_port.state, ConnectionState::Established);

        // Test development port classification
        assert!(listening_port.is_development_port()); // 8080 is development
        assert!(!established_port.is_development_port()); // 443 is not development
    }
}
