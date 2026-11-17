//! Syscall dispatch: numbered entry points routed to kernel handlers.

/// A syscall number.
pub type SyscallNr = u64;

/// Arguments passed to a syscall (up to 6, SysV-AMD64 style).
pub type SyscallArgs = [u64; 6];

/// Return value / error from a syscall handler.
pub type SyscallResult = i64;

/// Error code returned to userspace (negative errno-style).
pub const EPERM: i64 = -1;
pub const ENOENT: i64 = -2;
pub const EINVAL: i64 = -22;
pub const ENOSYS: i64 = -38;

/// A syscall handler.
pub type SyscallHandler = fn(SyscallArgs) -> SyscallResult;

/// Syscall table entry metadata.
#[derive(Debug, Clone)]
pub struct SyscallEntry {
    pub nr: SyscallNr,
    pub name: &'static str,
    pub handler: SyscallHandler,
}

/// Errors from the syscall table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyscallError {
    /// Syscall number already registered.
    Duplicate(SyscallNr),
    /// Syscall number not registered.
    Unregistered(SyscallNr),
}

impl std::fmt::Display for SyscallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicate(n) => write!(f, "syscall {n} already registered"),
            Self::Unregistered(n) => write!(f, "syscall {n} not registered"),
        }
    }
}
impl std::error::Error for SyscallError {}

/// The kernel syscall table.
#[derive(Debug, Default)]
pub struct SyscallTable {
    entries: std::collections::BTreeMap<SyscallNr, SyscallEntry>,
}

impl SyscallTable {
    pub fn new() -> Self { Self::default() }

    /// Register a handler for `nr`.
    pub fn register(&mut self, nr: SyscallNr, name: &'static str, h: SyscallHandler) -> Result<(), SyscallError> {
        if self.entries.contains_key(&nr) {
            return Err(SyscallError::Duplicate(nr));
        }
        self.entries.insert(nr, SyscallEntry { nr, name, handler: h });
        Ok(())
    }

    /// Dispatch a syscall; returns ENOSYS for unregistered numbers.
    pub fn dispatch(&self, nr: SyscallNr, args: SyscallArgs) -> SyscallResult {
        match self.entries.get(&nr) {
            Some(e) => (e.handler)(args),
            None => ENOSYS,
        }
    }

    /// Look up an entry by number.
    pub fn get(&self, nr: SyscallNr) -> Option<&SyscallEntry> { self.entries.get(&nr) }
    /// Number of registered syscalls.
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    /// Iterate entries sorted by number.
    pub fn iter(&self) -> impl Iterator<Item = &SyscallEntry> { self.entries.values() }
}

// ---- built-in syscall handlers (kernel self-tests / minimal set) ----

/// `sys_write(fd, buf, len)` stub — records nothing, returns len.
pub fn sys_write(_args: SyscallArgs) -> SyscallResult {
    _args[2] as i64
}

/// `sys_getpid()` — returns the current pid placeholder.
pub fn sys_getpid(_args: SyscallArgs) -> SyscallResult { 1 }

/// `sys_exit(code)` — returns the exit code.
pub fn sys_exit(args: SyscallArgs) -> SyscallResult { args[0] as i64 }

/// `sys_nanosleep` stub.
pub fn sys_nanosleep(_args: SyscallArgs) -> SyscallResult { 0 }

#[cfg(test)]
mod tests {
    use super::*;

    fn h_double(args: SyscallArgs) -> SyscallResult { (args[0] * 2) as i64 }

    #[test]
    fn register_and_dispatch() {
        let mut t = SyscallTable::new();
        t.register(0, "read", sys_write).unwrap();
        t.register(39, "getpid", sys_getpid).unwrap();
        assert_eq!(t.dispatch(39, [0; 6]), 1);
        assert_eq!(t.dispatch(0, [1, 2, 3, 0, 0, 0]), 3);
        assert_eq!(t.dispatch(999, [0; 6]), ENOSYS);
    }

    #[test]
    fn duplicate_rejected() {
        let mut t = SyscallTable::new();
        t.register(1, "a", h_double).unwrap();
        assert_eq!(t.register(1, "b", h_double), Err(SyscallError::Duplicate(1)));
    }

    #[test]
    fn custom_handler() {
        let mut t = SyscallTable::new();
        t.register(42, "double", h_double).unwrap();
        assert_eq!(t.dispatch(42, [21, 0, 0, 0, 0, 0]), 42);
    }
}
