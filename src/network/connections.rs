use crate::network::Protocol;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub protocol: Protocol,
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

impl ConnectionInfo {
    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();

        if self.local_address.to_string().contains(&query)
            || self.remote_address.to_string().contains(&query)
        {
            return true;
        }

        if let Some(process_name) = &self.process_name {
            if process_name.to_lowercase().contains(&query) {
                return true;
            }
        }

        if let Some(pid) = self.pid {
            if pid.to_string().contains(&query) {
                return true;
            }
        }

        false
    }
}
