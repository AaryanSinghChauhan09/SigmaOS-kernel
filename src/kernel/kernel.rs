//! The kernel core: orchestrates process, memory, syscall, scheduler, VFS,
//! timer, and interrupt subsystems behind a single `Kernel` object.

use crate::kernel::memory::AddressSpace;
use crate::kernel::process::{Pid, ProcessTable};
use crate::kernel::scheduler::{RoundRobin, Scheduler};
use crate::kernel::state::{KernelInstance, KernelState};
use crate::kernel::syscall::{SyscallArgs, SyscallResult, SyscallTable};
use crate::kernel::timer::{Idt, TimerWheel};
use crate::kernel::vfs::MountTable;
use std::collections::BTreeMap;

/// The assembled kernel.
pub struct Kernel {
    pub instance: KernelInstance,
    pub procs: ProcessTable,
    pub syscalls: SyscallTable,
    pub mounts: MountTable,
    pub timers: TimerWheel,
    pub idt: Idt,
    scheduler: RoundRobin,
    /// Per-process address spaces.
    spaces: BTreeMap<Pid, AddressSpace>,
    /// Currently running pid.
    pub current: Option<Pid>,
}

/// Errors from kernel-level operations.
#[derive(Debug)]
pub enum KernelError {
    Proc(crate::kernel::process::ProcError),
    Syscall(crate::kernel::syscall::SyscallError),
    /// Kernel is not in a state that allows the operation.
    NotReady(KernelState),
    /// No runnable process.
    NoRunnableProcess,
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proc(e) => write!(f, "process error: {e}"),
            Self::Syscall(e) => write!(f, "syscall error: {e}"),
            Self::NotReady(s) => write!(f, "kernel not ready (state {s:?})"),
            Self::NoRunnableProcess => write!(f, "no runnable process"),
        }
    }
}
impl std::error::Error for KernelError {}
impl From<crate::kernel::process::ProcError> for KernelError {
    fn from(e: crate::kernel::process::ProcError) -> Self { Self::Proc(e) }
}

impl Kernel {
    /// Create a kernel with the given process-table capacity.
    pub fn new(max_procs: usize) -> Self {
        Self {
            instance: KernelInstance::new(),
            procs: ProcessTable::with_capacity(max_procs),
            syscalls: SyscallTable::new(),
            mounts: MountTable::new(),
            timers: TimerWheel::new(),
            idt: Idt::new(),
            scheduler: RoundRobin::new(),
            spaces: BTreeMap::new(),
            current: None,
        }
    }

    /// Boot: transition Booting -> Initialized.
    pub fn boot(&mut self) {
        self.instance.set_state(KernelState::Booting);
        // subsystem init happens here in a real kernel
        self.instance.set_state(KernelState::Initialized);
    }

    /// Spawn a process with a fresh address space; returns pid.
    pub fn spawn(&mut self, name: impl Into<String>, parent: Option<Pid>) -> Result<Pid, KernelError> {
        if !self.instance.is_operational() {
            return Err(KernelError::NotReady(self.instance.state));
        }
        let pid = self.procs.spawn(name, parent)?;
        self.spaces.insert(pid, AddressSpace::new());
        Ok(pid)
    }

    /// Mark a process runnable and enqueue it.
    pub fn make_ready(&mut self, pid: Pid) -> Result<(), KernelError> {
        self.procs.set_state(pid, crate::kernel::process::ProcessState::Ready)?;
        if let Some(p) = self.procs.get(pid) {
            self.scheduler.enqueue(crate::kernel::scheduler::SchedTask {
                pid, priority: p.priority, timeslice: p.timeslice,
            });
        }
        Ok(())
    }

    /// Context-switch: pick next runnable process.
    pub fn schedule(&mut self) -> Result<Pid, KernelError> {
        let next = self.scheduler.dequeue().ok_or(KernelError::NoRunnableProcess)?;
        if let Some(cur) = self.current {
            let _ = self.procs.set_state(cur, crate::kernel::process::ProcessState::Ready);
        }
        let _ = self.procs.set_state(next.pid, crate::kernel::process::ProcessState::Running);
        self.current = Some(next.pid);
        Ok(next.pid)
    }

    /// Syscall entry point: dispatch through the table for `current`.
    pub fn syscall_entry(&self, nr: u64, args: SyscallArgs) -> SyscallResult {
        self.syscalls.dispatch(nr, args)
    }

    /// Timer tick: advance uptime, fire due timers, reschedule.
    pub fn tick(&mut self) -> Vec<u64> {
        self.instance.tick();
        self.timers.tick()
    }

    /// Route an interrupt through the IDT.
    pub fn interrupt(&self, vec: u8, irq: u64) {
        self.idt.dispatch(vec, irq);
    }

    /// Kernel panic.
    pub fn panic(&mut self, msg: impl Into<String>) {
        self.instance.panic(msg);
    }

    /// Access a process's address space.
    pub fn space(&self, pid: Pid) -> Option<&AddressSpace> { self.spaces.get(&pid) }
    /// Mutable address space.
    pub fn space_mut(&mut self, pid: Pid) -> Option<&mut AddressSpace> { self.spaces.get_mut(&pid) }

    /// Read-only scheduler view for the heavy scheduler component.
    pub fn scheduler(&self) -> &RoundRobin { &self.scheduler }
}
