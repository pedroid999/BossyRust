use crate::process::{ProcessInfo, ProcessManager};
use std::time::{Duration, Instant};

pub struct ProcessMonitor {
    manager: ProcessManager,
    last_update: Instant,
    update_interval: Duration,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            manager: ProcessManager::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(1000), // 1 second default
        }
    }

    pub fn should_update(&self) -> bool {
        self.last_update.elapsed() >= self.update_interval
    }

    pub fn get_processes(&mut self) -> Vec<ProcessInfo> {
        if self.should_update() {
            self.manager.refresh();
            self.last_update = Instant::now();
        }
        self.manager.get_processes()
    }

    pub fn get_top_cpu_processes(&mut self, limit: usize) -> Vec<ProcessInfo> {
        if self.should_update() {
            self.manager.refresh();
            self.last_update = Instant::now();
        }
        let mut processes = self.manager.get_processes();
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        processes.truncate(limit);
        processes
    }

    pub fn get_top_memory_processes(&mut self, limit: usize) -> Vec<ProcessInfo> {
        if self.should_update() {
            self.manager.refresh();
            self.last_update = Instant::now();
        }
        let mut processes = self.manager.get_processes();
        processes.sort_by(|a, b| b.memory.cmp(&a.memory));
        processes.truncate(limit);
        processes
    }
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}
