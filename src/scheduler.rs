//! Process scheduler with efficient scheduling algorithms
//! Follows: Efficient Scheduling, Low Level Control, Deadlock Avoidance

use std::collections::VecDeque;

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Realtime = 4,
}

/// Process control block
#[derive(Debug, Clone)]
pub struct Process {
    pub id: u64,
    pub priority: Priority,
    pub state: ProcessState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Priority-based scheduler
pub struct Scheduler {
    ready_queue: VecDeque<Process>,
    current_process: Option<Process>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            ready_queue: VecDeque::new(),
            current_process: None,
        }
    }

    /// Add process to ready queue
    pub fn add_process(&mut self, process: Process) {
        let mut processes: Vec<_> = self.ready_queue.drain(..).collect();
        processes.push(process);
        processes.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.ready_queue = processes.into();
    }

    /// Schedule next process
    pub fn schedule(&mut self) -> Option<&Process> {
        if let Some(process) = self.ready_queue.pop_front() {
            self.current_process = Some(process.clone());
            self.current_process.as_ref()
        } else {
            None
        }
    }

    /// Get current running process
    pub fn current_process(&self) -> Option<&Process> {
        self.current_process.as_ref()
    }

    /// Yield current process
    pub fn yield_process(&mut self) {
        if let Some(mut process) = self.current_process.take() {
            process.state = ProcessState::Ready;
            self.add_process(process);
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduling() {
        let mut scheduler = Scheduler::new();
        scheduler.add_process(Process {
            id: 1,
            priority: Priority::Normal,
            state: ProcessState::Ready,
        });
        scheduler.add_process(Process {
            id: 2,
            priority: Priority::High,
            state: ProcessState::Ready,
        });
        
        let scheduled = scheduler.schedule().unwrap();
        assert_eq!(scheduled.id, 2); // High priority should run first
    }
}
