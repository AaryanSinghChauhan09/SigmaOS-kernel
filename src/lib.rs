// SPDX-License-Identifier: Apache-2.0
//! SigmaOS kernel core — microkernel subsystems.
//!
//! Orchestrates process management, virtual memory, syscall dispatch,
//! scheduling, VFS mounts, timers, and interrupts behind a single `Kernel`.
//! Design principles: single responsibility, fail-fast, secure defaults,
//! mechanism vs policy separation.

#![forbid(unsafe_code)]

pub mod kernel;

pub use kernel::kernel::{Kernel, KernelError};
pub use kernel::memory::{AddressSpace, MemError, MemoryRegion, Prot, VirtAddr};
pub use kernel::process::{Pid, Process, ProcessState, ProcessTable, ProcError};
pub use kernel::scheduler::{enqueue_ready, RoundRobin, Scheduler, SchedError, SchedTask};
pub use kernel::state::{KernelInstance, KernelState};
pub use kernel::syscall::{
    SyscallArgs, SyscallEntry, SyscallError, SyscallNr, SyscallResult, SyscallTable,
};
pub use kernel::timer::{Idt, Tick, Timer, TimerError, TimerId, TimerWheel};
pub use kernel::vfs::{FileHandle, FileMeta, FileSystem, MemFs, Mount, MountTable, Path, VfsError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_instance_lifecycle() {
        let mut k = KernelInstance::new();
        assert!(k.is_operational());
        k.tick();
        assert_eq!(k.uptime_ticks, 1);
        k.panic("test");
        assert_eq!(k.state, KernelState::Panic);
    }

    #[test]
    fn kernel_boot_spawn_schedule() {
        let mut k = Kernel::new(16);
        k.boot();
        let pid = k.spawn("init", None).unwrap();
        k.make_ready(pid).unwrap();
        assert_eq!(k.schedule().unwrap(), pid);
        assert_eq!(k.current, Some(pid));
    }

    #[test]
    fn kernel_syscall_and_tick() {
        let mut k = Kernel::new(8);
        k.boot();
        // register a syscall then dispatch
        k.syscalls.register(0, "write", crate::kernel::syscall::sys_write).unwrap();
        assert_eq!(k.syscall_entry(0, [1, 2, 3, 0, 0, 0]), 3);
        let fired = k.tick();
        assert!(fired.is_empty());
        assert_eq!(k.instance.uptime_ticks, 1);
    }

    #[test]
    fn process_exit_and_reap() {
        let mut t = ProcessTable::with_capacity(4);
        let p = t.spawn("child", None).unwrap();
        t.set_state(p, ProcessState::Ready).unwrap();
        t.exit(p, 0).unwrap();
        assert_eq!(t.reap(p).unwrap(), 0);
        assert!(t.get(p).is_none());
    }

    #[test]
    fn memory_map_translate() {
        let mut a = AddressSpace::new();
        a.map(0x1000, 8192, Prot::RW).unwrap();
        assert_eq!(a.mapped_bytes(), 8192);
        assert!(a.lookup(0x1000).is_some());
        assert!(a.lookup(0x3000).is_none());
        assert!(a.translate(0x1000).is_some());
    }
}
