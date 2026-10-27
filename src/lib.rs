//! SigmaOS Kernel Library
//! Core kernel components: architecture support, memory management, scheduler, syscall dispatch, signal handling
//!
//! Principles:
//! - MINIMAL KERNEL PHILOSOPHY: Only essential kernel functionality
//! - MEMORY SAFETY: Safe Rust with controlled unsafe blocks
//! - ZERO-COPY: Efficient data passing where possible
//! - FAIL-FAST: Early error detection and reporting
//! - DETERMINISTIC TESTING: Comprehensive test coverage

#![allow(dead_code)]

// Core kernel modules (flat structure for MINIMAL KERNEL PHILOSOPHY)
// Restoring modules with klib dependencies
pub mod architecture;
pub mod boot_foundations;
pub mod boot_to_userspace;
pub mod capabilities;
pub mod cgroup_v2;
pub mod cgroups;
pub mod interrupt;
pub mod memory;
pub mod namespaces;
pub mod panic_handler;
pub mod process;
pub mod roundrobin;
pub mod scheduler;
pub mod seccomp;
pub mod security;
pub mod signal;
pub mod syscall;
pub mod thread;
pub mod timekeeping;
pub mod timer_subsystem;
pub mod tss_ring3_user_mode;

// Core architecture-specific modules
pub mod core;

// External dependencies
extern crate sigmaos_klib as klib;

// Re-export key kernel interfaces for other repositories
pub use memory::sigma_buddy::SigmaBuddyAllocator;
pub use roundrobin::RoundRobinScheduler;
pub use scheduler::{Priority, ProcessState, ProcessTask, SchedulerPolicy};
