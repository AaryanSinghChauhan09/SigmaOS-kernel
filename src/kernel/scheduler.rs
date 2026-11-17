//! Scheduler interface + a simple round-robin / priority implementation.
//! The heavy CFS/EEVDF logic lives in the `sigmaos-scheduler` component; the
//! kernel core defines the contract schedulers plug into.

use crate::kernel::process::{Pid, ProcessState};
use std::collections::BTreeMap;

/// Scheduler-facing view of a runnable task.
#[derive(Debug, Clone, Copy)]
pub struct SchedTask {
    pub pid: Pid,
    pub priority: i32,
    pub timeslice: u64,
}

/// Errors from the scheduler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedError {
    /// PID not currently enqueued.
    NotQueued(Pid),
    /// Run queue is empty.
    Empty,
}

impl std::fmt::Display for SchedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotQueued(p) => write!(f, "pid {p} not queued"),
            Self::Empty => write!(f, "run queue empty"),
        }
    }
}
impl std::error::Error for SchedError {}

/// The scheduler contract the kernel uses.
pub trait Scheduler {
    /// Enqueue a ready task.
    fn enqueue(&mut self, task: SchedTask);
    /// Dequeue the next task to run.
    fn dequeue(&mut self) -> Option<SchedTask>;
    /// Pick (but don't remove) the next task.
    fn peek(&self) -> Option<SchedTask>;
    /// Remove a specific task (e.g. it blocked or exited).
    fn remove(&mut self, pid: Pid) -> Result<(), SchedError>;
    /// Number of queued tasks.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
}

/// Round-robin scheduler with optional priority ordering.
/// Tasks with equal priority run FIFO; higher priority preempts.
#[derive(Debug, Default)]
pub struct RoundRobin {
    queue: std::collections::VecDeque<SchedTask>,
    // index by pid for O(log n) removal bookkeeping
    present: BTreeMap<Pid, ()>,
}

impl RoundRobin {
    pub fn new() -> Self { Self::default() }

    /// Insert maintaining priority order (highest priority first, FIFO within).
    fn insert_sorted(&mut self, task: SchedTask) {
        let pos = self.queue.iter().position(|t| t.priority < task.priority)
            .unwrap_or(self.queue.len());
        self.queue.insert(pos, task);
        self.present.insert(task.pid, ());
    }
}

impl Scheduler for RoundRobin {
    fn enqueue(&mut self, task: SchedTask) {
        if self.present.contains_key(&task.pid) { return; } // already queued
        self.insert_sorted(task);
    }

    fn dequeue(&mut self) -> Option<SchedTask> {
        let t = self.queue.pop_front()?;
        self.present.remove(&t.pid);
        Some(t)
    }

    fn peek(&self) -> Option<SchedTask> { self.queue.front().copied() }

    fn remove(&mut self, pid: Pid) -> Result<(), SchedError> {
        if self.present.remove(&pid).is_none() {
            return Err(SchedError::NotQueued(pid));
        }
        self.queue.retain(|t| t.pid != pid);
        Ok(())
    }

    fn len(&self) -> usize { self.queue.len() }
}

/// Bridge the process table to the scheduler: enqueue all Ready processes.
pub fn enqueue_ready(
    sched: &mut dyn Scheduler,
    procs: &crate::kernel::process::ProcessTable,
) {
    for p in procs.pids_in_state(ProcessState::Ready) {
        if let Some(proc_) = procs.get(p) {
            sched.enqueue(SchedTask { pid: p, priority: proc_.priority, timeslice: proc_.timeslice });
        }
    }
}
