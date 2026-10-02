use sysinfo::{Pid, ProcessesToUpdate, System};

pub fn process_memory_mb() -> Option<u64> {
    let pid = Pid::from_u32(std::process::id());
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
    let process = system.process(pid)?;
    Some(process.memory() / 1024 / 1024)
}
