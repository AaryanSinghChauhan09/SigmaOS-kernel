//! SigmaOS kernel core subsystems.

pub mod kernel;
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod state;
pub mod syscall;
pub mod timer;
pub mod vfs;

pub use kernel::{Kernel, KernelError};
pub use memory::{AddressSpace, MemError, MemoryRegion, Prot, VirtAddr};
pub use process::{Pid, Process, ProcessState, ProcessTable, ProcError};
pub use scheduler::{enqueue_ready, RoundRobin, Scheduler, SchedError, SchedTask};
pub use state::{KernelInstance, KernelState};
pub use syscall::{SyscallArgs, SyscallEntry, SyscallError, SyscallNr, SyscallResult, SyscallTable};
pub use timer::{Idt, Tick, Timer, TimerError, TimerId, TimerWheel};
pub use vfs::{FileHandle, FileMeta, FileSystem, MemFs, Mount, MountTable, Path, VfsError};
