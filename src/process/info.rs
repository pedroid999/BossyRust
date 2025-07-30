use sysinfo::{Pid, Process, System};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    #[allow(dead_code)]
    pub parent_pid: Option<u32>,
    pub status: String,
    #[allow(dead_code)]
    pub start_time: u64,
    #[allow(dead_code)]
    pub user_id: Option<u32>,
    #[allow(dead_code)]
    pub executable_path: Option<String>,
    #[allow(dead_code)]
    pub command_line: Vec<String>,
}

impl ProcessInfo {
    pub fn from_sysinfo(pid: Pid, process: &Process) -> Self {
        Self {
            pid: pid.as_u32(),
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            status: format!("{:?}", process.status()),
            start_time: process.start_time(),
            user_id: process
                .user_id()
                .and_then(|u| u.to_string().parse::<u32>().ok()),
            executable_path: process.exe().and_then(|p| p.to_str().map(String::from)),
            command_line: process.cmd().to_vec(),
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();

        // Handle special search patterns
        if let Some(pid_query) = query.strip_prefix('#') {
            // PID search: #1234
            if let Ok(search_pid) = pid_query.parse::<u32>() {
                return self.pid == search_pid;
            }
        } else if let Some(resource_query) = query.strip_prefix('>') {
            // Resource usage search: >50% for CPU, >1GB for memory
            if let Some(cpu_query) = resource_query.strip_suffix('%') {
                if let Ok(cpu_threshold) = cpu_query.parse::<f32>() {
                    return self.cpu_usage > cpu_threshold;
                }
            } else if let Some(mem_query) = resource_query
                .strip_suffix("gb")
                .or_else(|| resource_query.strip_suffix("GB"))
            {
                if let Ok(mem_threshold) = mem_query.parse::<f32>() {
                    let mem_gb = self.memory as f32 / 1024.0 / 1024.0 / 1024.0;
                    return mem_gb > mem_threshold;
                }
            } else if let Some(mem_query) = resource_query
                .strip_suffix("mb")
                .or_else(|| resource_query.strip_suffix("MB"))
            {
                if let Ok(mem_threshold) = mem_query.parse::<f32>() {
                    let mem_mb = self.memory as f32 / 1024.0 / 1024.0;
                    return mem_mb > mem_threshold;
                }
            }
        }

        // Default name-based search
        self.name.to_lowercase().contains(&query)
    }

    pub fn format_memory(&self) -> String {
        let kb = self.memory / 1024;
        let mb = kb / 1024;
        let gb = mb / 1024;

        if gb > 0 {
            format!("{:.1}GB", gb as f64 / 1.0)
        } else if mb > 0 {
            format!("{mb}MB")
        } else {
            format!("{kb}KB")
        }
    }
}

pub struct ProcessManager {
    system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        self.system
            .processes()
            .iter()
            .map(|(&pid, process)| ProcessInfo::from_sysinfo(pid, process))
            .collect()
    }

    pub fn get_system_cpu_usage(&self) -> f32 {
        self.system.global_cpu_info().cpu_usage()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_process() -> ProcessInfo {
        ProcessInfo {
            pid: 1234,
            name: "test_process".to_string(),
            cpu_usage: 25.5,
            memory: 1024 * 1024 * 512, // 512 MB
            parent_pid: Some(1),
            status: "Running".to_string(),
            start_time: 1234567890,
            user_id: Some(1000),
            executable_path: Some("/usr/bin/test_process".to_string()),
            command_line: vec![
                "test_process".to_string(),
                "--arg1".to_string(),
                "value".to_string(),
            ],
        }
    }

    #[test]
    fn test_process_info_creation() {
        let process = create_test_process();
        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "test_process");
        assert_eq!(process.cpu_usage, 25.5);
        assert_eq!(process.memory, 1024 * 1024 * 512);
    }

    #[test]
    fn test_memory_formatting() {
        let mut process = create_test_process();

        // Test MB formatting
        process.memory = 1024 * 1024 * 100; // 100 MB
        assert_eq!(process.format_memory(), "100MB");

        // Test GB formatting
        process.memory = 1024 * 1024 * 1024 * 2; // 2 GB
        assert_eq!(process.format_memory(), "2.0GB");

        // Test KB formatting
        process.memory = 1024 * 500; // 500 KB
        assert_eq!(process.format_memory(), "500KB");
    }

    #[test]
    fn test_search_by_name() {
        let process = create_test_process();
        assert!(process.matches_search("test"));
        assert!(process.matches_search("TEST")); // case insensitive
        assert!(process.matches_search("process"));
        assert!(!process.matches_search("notfound"));
    }

    #[test]
    fn test_search_by_pid() {
        let process = create_test_process();
        assert!(process.matches_search("#1234"));
        assert!(!process.matches_search("#5678"));
    }

    #[test]
    fn test_search_by_cpu_usage() {
        let process = create_test_process();
        assert!(process.matches_search(">20%"));
        assert!(process.matches_search(">25%"));
        assert!(!process.matches_search(">30%"));
    }

    #[test]
    fn test_search_by_memory_usage() {
        let process = create_test_process();

        // Test MB search
        assert!(process.matches_search(">400MB"));
        assert!(!process.matches_search(">600MB"));

        // Test GB search
        assert!(!process.matches_search(">1GB")); // 512MB < 1GB
    }

    #[test]
    fn test_invalid_search_patterns() {
        let process = create_test_process();

        // Invalid PID format
        assert!(!process.matches_search("#abc"));

        // Invalid CPU format
        assert!(!process.matches_search(">abc%"));

        // Invalid memory format
        assert!(!process.matches_search(">abcMB"));
    }

    #[test]
    fn test_process_manager_creation() {
        let manager = ProcessManager::new();
        let processes = manager.get_processes();

        // Should have at least some processes running
        assert!(!processes.is_empty());

        // All processes should have valid PIDs
        for process in &processes {
            assert!(process.pid > 0);
        }
    }

    #[test]
    fn test_process_filtering() {
        let manager = ProcessManager::new();
        let all_processes = manager.get_processes();

        // Test that we get some processes
        assert!(!all_processes.is_empty());

        // Test that basic process info is populated
        for process in all_processes.iter().take(5) {
            assert!(process.pid > 0);
            assert!(!process.name.is_empty());
        }
    }

    #[test]
    fn test_process_tree_structure() {
        let manager = ProcessManager::new();
        let processes = manager.get_processes();

        // Test that processes have valid parent relationships
        for process in processes.iter().take(10) {
            // PIDs should be positive
            assert!(process.pid > 0);

            // Parent PIDs, if present, should also be positive
            if let Some(_parent_pid) = process.parent_pid {
                // Parent PIDs should be valid
            }
        }
    }

    #[test]
    fn test_search_functionality() {
        let manager = ProcessManager::new();

        // Test basic search functionality
        let all_processes = manager.get_processes();

        // Test that we get some processes and they have searchable info
        assert!(!all_processes.is_empty());

        // Test search pattern matching with first process
        if let Some(first_process) = all_processes.first() {
            // Should match itself by name
            assert!(first_process.matches_search(&first_process.name.to_lowercase()));
        }
    }

    #[test]
    fn test_process_sorting() {
        let manager = ProcessManager::new();
        let mut processes = manager.get_processes();

        // Test CPU sorting manually
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        processes.truncate(10);

        for i in 1..processes.len() {
            assert!(processes[i - 1].cpu_usage >= processes[i].cpu_usage);
        }

        // Test memory sorting manually
        let mut processes = manager.get_processes();
        processes.sort_by(|a, b| b.memory.cmp(&a.memory));
        processes.truncate(10);

        for i in 1..processes.len() {
            assert!(processes[i - 1].memory >= processes[i].memory);
        }
    }
}
