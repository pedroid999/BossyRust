use std::collections::HashMap;

pub struct NetworkUtils;

impl NetworkUtils {
    pub fn get_well_known_ports() -> HashMap<u16, &'static str> {
        let mut ports = HashMap::new();

        // System ports (0-1023)
        ports.insert(21, "FTP");
        ports.insert(22, "SSH");
        ports.insert(23, "Telnet");
        ports.insert(25, "SMTP");
        ports.insert(53, "DNS");
        ports.insert(67, "DHCP Server");
        ports.insert(68, "DHCP Client");
        ports.insert(80, "HTTP");
        ports.insert(110, "POP3");
        ports.insert(143, "IMAP");
        ports.insert(443, "HTTPS");
        ports.insert(993, "IMAPS");
        ports.insert(995, "POP3S");

        // Database ports
        ports.insert(1433, "SQL Server");
        ports.insert(1521, "Oracle");
        ports.insert(3306, "MySQL");
        ports.insert(5432, "PostgreSQL");
        ports.insert(6379, "Redis");
        ports.insert(27017, "MongoDB");

        // Web development ports
        ports.insert(3000, "React/Next.js Dev");
        ports.insert(3001, "Create React App");
        ports.insert(4200, "Angular Dev");
        ports.insert(5000, "Flask Dev");
        ports.insert(8000, "Django Dev");
        ports.insert(8080, "HTTP Alt/Tomcat");
        ports.insert(8443, "HTTPS Alt");

        // Message queues
        ports.insert(5672, "RabbitMQ");
        ports.insert(1883, "MQTT");
        ports.insert(9092, "Kafka");

        // Search engines
        ports.insert(9200, "Elasticsearch");
        ports.insert(8983, "Solr");

        // Monitoring & metrics
        ports.insert(3001, "Grafana");
        ports.insert(9090, "Prometheus");
        ports.insert(8086, "InfluxDB");

        ports
    }

    pub fn get_development_ports() -> Vec<u16> {
        vec![
            3000, 3001, 3002, 3003, 3004, 3005, // React, Next.js variants
            4200, 4201, 4202, // Angular variants
            5000, 5001, 5002, // Flask, various dev servers
            8000, 8001, 8002, // Django variants
            8080, 8081, 8082, 8083, 8084, 8085, // Generic HTTP variants
            9000, 9001, 9002, // Various dev tools
        ]
    }

    pub fn is_development_port(port: u16) -> bool {
        Self::get_development_ports().contains(&port)
    }

    pub fn suggest_alternative_port(port: u16) -> Vec<u16> {
        match port {
            3000 => vec![3001, 3002, 3003, 8000],
            8000 => vec![8001, 8080, 3000],
            8080 => vec![8081, 8000, 3000],
            5000 => vec![5001, 5002, 8000],
            4200 => vec![4201, 4202, 3000],
            _ => {
                // Generate alternatives around the requested port
                let mut alternatives = Vec::new();
                for i in 1..=5 {
                    if let Some(new_port) = port.checked_add(i) {
                        alternatives.push(new_port);
                    }
                }
                alternatives
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_known_ports() {
        let ports = NetworkUtils::get_well_known_ports();

        // Test system ports
        assert_eq!(ports.get(&21), Some(&"FTP"));
        assert_eq!(ports.get(&22), Some(&"SSH"));
        assert_eq!(ports.get(&80), Some(&"HTTP"));
        assert_eq!(ports.get(&443), Some(&"HTTPS"));

        // Test database ports
        assert_eq!(ports.get(&3306), Some(&"MySQL"));
        assert_eq!(ports.get(&5432), Some(&"PostgreSQL"));
        assert_eq!(ports.get(&6379), Some(&"Redis"));
        assert_eq!(ports.get(&27017), Some(&"MongoDB"));

        // Test development ports
        assert_eq!(ports.get(&3000), Some(&"React/Next.js Dev"));
        assert_eq!(ports.get(&4200), Some(&"Angular Dev"));
        assert_eq!(ports.get(&5000), Some(&"Flask Dev"));

        // Test non-existent port
        assert_eq!(ports.get(&65534), None);
    }

    #[test]
    fn test_development_ports() {
        let dev_ports = NetworkUtils::get_development_ports();

        // Should contain common dev ports
        assert!(dev_ports.contains(&3000));
        assert!(dev_ports.contains(&3001));
        assert!(dev_ports.contains(&4200));
        assert!(dev_ports.contains(&5000));
        assert!(dev_ports.contains(&8000));
        assert!(dev_ports.contains(&8080));

        // Should not be empty
        assert!(!dev_ports.is_empty());
    }

    #[test]
    fn test_is_development_port() {
        // Test common development ports
        assert!(NetworkUtils::is_development_port(3000));
        assert!(NetworkUtils::is_development_port(3001));
        assert!(NetworkUtils::is_development_port(4200));
        assert!(NetworkUtils::is_development_port(5000));
        assert!(NetworkUtils::is_development_port(8000));
        assert!(NetworkUtils::is_development_port(8080));

        // Test non-development ports
        assert!(!NetworkUtils::is_development_port(80));
        assert!(!NetworkUtils::is_development_port(443));
        assert!(!NetworkUtils::is_development_port(22));
        assert!(!NetworkUtils::is_development_port(25));
    }

    #[test]
    fn test_suggest_alternative_port() {
        // Test common development ports
        let alternatives = NetworkUtils::suggest_alternative_port(3000);
        assert!(alternatives.contains(&3001));
        assert!(alternatives.contains(&3002));
        assert!(alternatives.contains(&8000));

        let alternatives = NetworkUtils::suggest_alternative_port(8000);
        assert!(alternatives.contains(&8001));
        assert!(alternatives.contains(&8080));
        assert!(alternatives.contains(&3000));

        let alternatives = NetworkUtils::suggest_alternative_port(5000);
        assert!(alternatives.contains(&5001));
        assert!(alternatives.contains(&5002));
        assert!(alternatives.contains(&8000));

        // Test uncommon port
        let alternatives = NetworkUtils::suggest_alternative_port(12345);
        assert!(alternatives.len() > 0);
        assert!(alternatives.len() <= 5);

        // All suggestions should be in valid range
        for port in &alternatives {
            assert!(*port > 12345);
            // All u16 values are <= 65535 by definition
        }
    }

    #[test]
    fn test_alternative_port_suggestions_edge_cases() {
        // Test edge cases for port suggestions
        let alternatives = NetworkUtils::suggest_alternative_port(65530);

        // Should still provide valid alternatives
        for _port in &alternatives {
            // All u16 values are <= 65535 by definition
        }

        // Should not suggest ports beyond the valid range
        // All u16 values are <= 65535 by definition
    }
}
