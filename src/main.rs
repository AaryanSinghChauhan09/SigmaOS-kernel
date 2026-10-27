#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
#![allow(clippy::all, unused)]
use std::string::String;

// SigmaOS Kernel Main Entry Point

use core::sync::atomic::{AtomicUsize, Ordering};
use std::string::ToString;

// Use library modules
use sigmaos_kernel::memory::sigma_buddy::SigmaBuddyAllocator as MemoryAllocator;
use sigmaos_kernel::roundrobin::RoundRobinScheduler as Scheduler;
use sigmaos_kernel::scheduler::ProcessState as TaskProcessState;
use sigmaos_kernel::scheduler::{Priority, ProcessTask, SchedulerPolicy};

pub static EARLY_CPU_STATE: core::sync::atomic::AtomicUsize = AtomicUsize::new(0);

/// The supreme x86_64 start_kernel bootstrap manager (similar to Linux's init/main.c)
pub fn start_kernel(
    _allocator: &mut MemoryAllocator,
    scheduler: &mut Scheduler,
) -> Result<(), &'static str> {
    // Stage 1: CPU hardware bootstrap (cli, cld, GDT, IDT, Paging)
    // TODO: Implement x86_64 initialization in architecture module
    EARLY_CPU_STATE.store(1, Ordering::SeqCst);

    // Stage 2: Physical Memory paging bootstrap
    // Allocator is already initialized in _start
    // TODO: Implement VMM when memory subsystem is complete
    EARLY_CPU_STATE.store(2, Ordering::SeqCst);

    // Stage 3: Scheduler bootstrap
    let idle_proc = ProcessTask::new_simple(0, Priority::IDLE);
    let _ = scheduler.add_process(idle_proc);

    // Stage 4: Open early userland runlevels (OpenRC)
    // TODO: Implement OpenRC compatibility layer in separate repository
    // when userland subsystems are complete

    EARLY_CPU_STATE.store(3, Ordering::SeqCst);
    Ok(())
}

#[cfg(target_os = "none")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut allocator = MemoryAllocator::new(0x10000, 1024 * 1024, 10);
    let mut scheduler = Scheduler::new();

    let _ = start_kernel(&mut allocator, &mut scheduler);

    loop {}
}

#[cfg(not(target_os = "none"))]
fn main() {
    // Hosted dummy entry point
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_64_start_kernel_bootstrap() {
        let mut allocator = MemoryAllocator::new(0x10000, 1024 * 1024, 10);
        let mut scheduler = Scheduler::new();

        assert_eq!(EARLY_CPU_STATE.load(Ordering::SeqCst), 0);

        start_kernel(&mut allocator, &mut scheduler).unwrap();

        assert_eq!(EARLY_CPU_STATE.load(Ordering::SeqCst), 3);
    }
}
