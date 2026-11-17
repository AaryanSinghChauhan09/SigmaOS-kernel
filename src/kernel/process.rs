//! Process table: PID allocation, lifecycle states, parent/child links.

use std::collections::BTreeMap;

/// Process ID (kernel-global, nonzero).
pub type Pid = u32;

/// Process lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessState {
    /// Being created, not yet runnable.
    New,
    /// Ready to be scheduled.
    Ready,
    /// Currently on-CPU.
    Running,
    /// Blocked on I/O / event.
    Blocked,
    /// Sleeping (interruptible).
    Sleeping,
    /// Exited, awaiting reap.
    Zombie,
    /// Killed / dead.
    Dead,
}

/// A kernel process (task) control block.
#[derive(Debug, Clone)]
pub struct Process {
    pub pid: Pid,
    pub ppid: Pid,
    pub state: ProcessState,
    pub name: String,
    /// Children pids.
    pub children: Vec<Pid>,
    /// Scheduler priority hint (higher = sooner).
    pub priority: i32,
    /// Remaining timeslice (scheduler-managed).
    pub timeslice: u64,
    /// Exit code once Zombie/Dead.
    pub exit_code: i32,
}

impl Process {
    pub fn new(pid: Pid, name: impl Into<String>) -> Self {
        Self {
            pid,
            ppid: 0,
            state: ProcessState::New,
            name: name.into(),
            children: Vec::new(),
            priority: 0,
            timeslice: 0,
            exit_code: 0,
        }
    }
}

/// Errors from process-table operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcError {
    /// PID already in use.
    PidExists(Pid),
    /// PID not found.
    NoSuchPid(Pid),
    /// Process is not in a state that allows the operation.
    BadState(Pid, ProcessState),
    /// Table is full.
    Full,
}

impl std::fmt::Display for ProcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PidExists(p) => write!(f, "pid {p} already exists"),
            Self::NoSuchPid(p) => write!(f, "no such pid {p}"),
            Self::BadState(p, s) => write!(f, "pid {p} in bad state {s:?}"),
            Self::Full => write!(f, "process table full"),
        }
    }
}
impl std::error::Error for ProcError {}

/// The kernel process table.
#[derive(Debug, Default)]
pub struct ProcessTable {
    procs: BTreeMap<Pid, Process>,
    next_pid: Pid,
    max: usize,
}

impl ProcessTable {
    /// Create a table with room for `max` processes.
    pub fn with_capacity(max: usize) -> Self {
        Self { procs: BTreeMap::new(), next_pid: 1, max }
    }

    pub fn len(&self) -> usize { self.procs.len() }
    pub fn is_empty(&self) -> bool { self.procs.is_empty() }

    /// Spawn a new process; returns its pid. Starts in `New`, caller moves to Ready.
    pub fn spawn(&mut self, name: impl Into<String>, parent: Option<Pid>) -> Result<Pid, ProcError> {
        if self.procs.len() >= self.max {
            return Err(ProcError::Full);
        }
        let pid = self.alloc_pid()?;
        let mut p = Process::new(pid, name);
        if let Some(pp) = parent {
            if let Some(par) = self.procs.get_mut(&pp) {
                p.ppid = pp;
                par.children.push(pid);
            }
        }
        self.procs.insert(pid, p);
        Ok(pid)
    }

    fn alloc_pid(&mut self) -> Result<Pid, ProcError> {
        loop {
            let pid = self.next_pid;
            self.next_pid = self.next_pid.wrapping_add(1).max(1);
            if !self.procs.contains_key(&pid) {
                return Ok(pid);
            }
            if self.next_pid == 1 {
                // wrapped all the way around
                return Err(ProcError::Full);
            }
        }
    }

    /// Look up a process.
    pub fn get(&self, pid: Pid) -> Option<&Process> { self.procs.get(&pid) }
    /// Mutable lookup.
    pub fn get_mut(&mut self, pid: Pid) -> Option<&mut Process> { self.procs.get_mut(&pid) }

    /// Transition a process to a new state.
    pub fn set_state(&mut self, pid: Pid, s: ProcessState) -> Result<(), ProcError> {
        let p = self.procs.get_mut(&pid).ok_or(ProcError::NoSuchPid(pid))?;
        p.state = s;
        Ok(())
    }

    /// Exit a process: mark Zombie (or Dead) and record code.
    pub fn exit(&mut self, pid: Pid, code: i32) -> Result<(), ProcError> {
        let p = self.procs.get_mut(&pid).ok_or(ProcError::NoSuchPid(pid))?;
        p.exit_code = code;
        p.state = ProcessState::Zombie;
        Ok(())
    }

    /// Reap a zombie, returning its exit code and detaching from parent.
    pub fn reap(&mut self, pid: Pid) -> Result<i32, ProcError> {
        let p = self.procs.get(&pid).ok_or(ProcError::NoSuchPid(pid))?;
        if p.state != ProcessState::Zombie {
            return Err(ProcError::BadState(pid, p.state));
        }
        let code = p.exit_code;
        let ppid = p.ppid;
        self.procs.remove(&pid);
        if let Some(par) = self.procs.get_mut(&ppid) {
            par.children.retain(|&c| c != pid);
        }
        Ok(code)
    }

    /// Iterate all processes.
    pub fn iter(&self) -> impl Iterator<Item = &Process> { self.procs.values() }
    /// Pids of processes in a given state.
    pub fn pids_in_state(&self, s: ProcessState) -> Vec<Pid> {
        self.procs.values().filter(|p| p.state == s).map(|p| p.pid).collect()
    }
}
