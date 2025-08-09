use std::time::Duration;

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System, Pid};

#[derive(Copy, Clone)]
pub enum SortBy {
    Cpu,
    Mem,
    Pid,
}

pub struct App {
    pub sys: System,
    pub sort_by: SortBy,
    pub filter: String,
    pub is_filtering: bool,
    pub show_help: bool,
    pub selected_index: usize,
    pub selected_pid: Option<Pid>,
    pub tick_rate: Duration,
}

impl App {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        sys.refresh_all();
        Self {
            sys,
            sort_by: SortBy::Cpu,
            filter: String::new(),
            is_filtering: false,
            show_help: false,
            selected_index: 0,
            selected_pid: None,
            tick_rate: Duration::from_millis(500),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
    }
}


