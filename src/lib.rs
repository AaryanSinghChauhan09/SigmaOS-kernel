//! SigmaOS Kernel Library
//! Core kernel components: architecture support, memory management, scheduler, syscall dispatch, signal handling
//!
//! Principles:
//! - MINIMAL KERNEL PHILOSOPHY: Only essential kernel functionality
//! - MEMORY SAFETY: Safe Rust with controlled unsafe blocks
//! - ZERO-COPY: Efficient data passing where possible
//! - FAIL-FAST: Early error detection and reporting
//! - DETERMINISTIC TESTING: Comprehensive test coverage

#![no_std] // Core kernel runs without standard library
#![allow(dead_code)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

pub mod arch;
pub mod boot;
pub mod interrupt;
pub mod kernel;
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod signal;
pub mod syscall;
pub mod thread;

// Re-export key kernel interfaces for other repositories
pub use arch::Arch;
pub use memory::PhysicalMemoryManager;
pub use scheduler::Scheduler;
pub use syscall::SyscallDispatcher;
