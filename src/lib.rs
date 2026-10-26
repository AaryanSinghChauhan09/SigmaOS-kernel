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
// Temporarily commented out modules with klib dependencies
pub mod architecture;
// pub mod scheduler;
// pub mod process;
// pub mod thread;
// pub mod signal;
// pub mod memory;
// pub mod syscall;
// pub mod interrupt;
// pub mod boot_foundations;
// pub mod boot_to_userspace;
// pub mod panic_handler;
// pub mod timekeeping;
// pub mod timer_subsystem;
// pub mod cgroups;
// pub mod cgroup_v2;
// pub mod namespaces;
// pub mod capabilities;
// pub mod seccomp;
// pub mod security;

// Core architecture-specific modules
pub mod core;

// Re-export key kernel interfaces for other repositories
// Commented out until compilation issues are resolved
// pub use architecture::Architecture;
