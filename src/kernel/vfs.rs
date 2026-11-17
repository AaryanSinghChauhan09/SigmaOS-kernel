//! VFS interface + mount table. Concrete filesystems live in the
//! `sigmaos-vfs`/`sigmaos-fs` components; the kernel core defines the contract.

use std::collections::BTreeMap;

/// Opaque file handle.
pub type FileHandle = u64;
/// A path in the unified namespace.
pub type Path = String;

/// Filesystem operation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VfsError {
    /// Path not found.
    NotFound(Path),
    /// Permission denied.
    PermissionDenied(Path),
    /// Filesystem is read-only.
    ReadOnly(Path),
    /// Invalid argument.
    Invalid(&'static str),
    /// I/O error.
    Io(String),
}

impl std::fmt::Display for VfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "not found: {p}"),
            Self::PermissionDenied(p) => write!(f, "permission denied: {p}"),
            Self::ReadOnly(p) => write!(f, "read-only: {p}"),
            Self::Invalid(m) => write!(f, "invalid: {m}"),
            Self::Io(m) => write!(f, "io error: {m}"),
        }
    }
}
impl std::error::Error for VfsError {}

/// The filesystem contract.
pub trait FileSystem {
    /// Filesystem type name (e.g. "ext4", "tmpfs", "proc").
    fn name(&self) -> &'static str;
    /// Read a whole file.
    fn read(&self, path: &Path) -> Result<Vec<u8>, VfsError>;
    /// Write a whole file (create/truncate).
    fn write(&self, path: &Path, data: &[u8]) -> Result<(), VfsError>;
    /// List directory entries.
    fn list(&self, path: &Path) -> Result<Vec<String>, VfsError>;
    /// Create a directory.
    fn mkdir(&self, path: &Path) -> Result<(), VfsError>;
    /// Remove a file or empty directory.
    fn remove(&self, path: &Path) -> Result<(), VfsError>;
    /// Stat-like metadata.
    fn stat(&self, path: &Path) -> Result<FileMeta, VfsError>;
}

/// Minimal file metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMeta {
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub readonly: bool,
}

/// A mounted filesystem at a mountpoint.
pub struct Mount {
    pub mountpoint: Path,
    pub fs_name: String,
    /// Length-prefix priority for longest-match routing (higher = more specific).
    pub fs: Box<dyn FileSystem>,
}

/// The VFS mount table routing paths to filesystems.
#[derive(Default)]
pub struct MountTable {
    mounts: Vec<Mount>,
}

impl MountTable {
    pub fn new() -> Self { Self::default() }

    /// Mount `fs` at `mountpoint`.
    pub fn mount(&mut self, mountpoint: impl Into<String>, fs: Box<dyn FileSystem>) {
        let mp = normalize(&mountpoint.into());
        self.mounts.retain(|m| m.mountpoint != mp); // replace existing
        self.mounts.push(Mount { mountpoint: mp, fs_name: fs.name().to_string(), fs });
        // longest mountpoint first for correct routing
        self.mounts.sort_by(|a, b| b.mountpoint.len().cmp(&a.mountpoint.len()));
    }

    /// Unmount the filesystem at `mountpoint`.
    pub fn unmount(&mut self, mountpoint: &str) -> bool {
        let mp = normalize(mountpoint);
        let n = self.mounts.len();
        self.mounts.retain(|m| m.mountpoint != mp);
        self.mounts.len() != n
    }

    /// Find the filesystem + relative path for a full path (longest-prefix match).
    fn resolve<'a>(&'a self, path: &str) -> Option<(&'a Mount, String)> {
        let p = normalize(path);
        for m in &self.mounts {
            if p == m.mountpoint {
                return Some((m, "/".to_string()));
            }
            if m.mountpoint == "/" {
                // root mount catches everything; fs sees the full path
                return Some((m, p.clone()));
            }
            if let Some(rest) = p.strip_prefix(&format!("{}/", m.mountpoint)) {
                return Some((m, format!("/{rest}")));
            }
        }
        None
    }

    /// Read through the VFS.
    pub fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let (m, rel) = self.resolve(path).ok_or_else(|| VfsError::NotFound(path.into()))?;
        m.fs.read(&rel)
    }
    /// Write through the VFS.
    pub fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError> {
        let (m, rel) = self.resolve(path).ok_or_else(|| VfsError::NotFound(path.into()))?;
        m.fs.write(&rel, data)
    }
    /// List a directory through the VFS.
    pub fn list(&self, path: &str) -> Result<Vec<String>, VfsError> {
        let (m, rel) = self.resolve(path).ok_or_else(|| VfsError::NotFound(path.into()))?;
        m.fs.list(&rel)
    }
    /// Stat through the VFS.
    pub fn stat(&self, path: &str) -> Result<FileMeta, VfsError> {
        let (m, rel) = self.resolve(path).ok_or_else(|| VfsError::NotFound(path.into()))?;
        m.fs.stat(&rel)
    }

    /// List mounts.
    pub fn mounts(&self) -> impl Iterator<Item = (&str, &str)> {
        self.mounts.iter().map(|m| (m.mountpoint.as_str(), m.fs_name.as_str()))
    }
    pub fn len(&self) -> usize { self.mounts.len() }
    pub fn is_empty(&self) -> bool { self.mounts.is_empty() }
}

fn normalize(p: &str) -> String {
    let mut out = String::from("/");
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => { out.pop(); if !out.ends_with('/') { out.push('/'); } }
            s => { if !out.ends_with('/') { out.push('/'); } out.push_str(s); }
        }
    }
    if out.len() > 1 && out.ends_with('/') { out.pop(); }
    if out.is_empty() { out.push('/'); }
    out
}

// ---- in-memory test filesystem ----

/// A simple in-memory filesystem (useful for tests and initramfs).
/// Uses interior mutability so it satisfies the `&self` filesystem contract.
#[derive(Debug, Default)]
pub struct MemFs {
    files: std::cell::RefCell<BTreeMap<String, Vec<u8>>>,
    dirs: std::cell::RefCell<std::collections::BTreeSet<String>>,
}

impl MemFs {
    pub fn new() -> Self { Self::default() }
}

impl FileSystem for MemFs {
    fn name(&self) -> &'static str { "memfs" }
    fn read(&self, path: &Path) -> Result<Vec<u8>, VfsError> {
        self.files.borrow().get(path).cloned().ok_or_else(|| VfsError::NotFound(path.clone()))
    }
    fn write(&self, path: &Path, data: &[u8]) -> Result<(), VfsError> {
        self.files.borrow_mut().insert(path.clone(), data.to_vec());
        Ok(())
    }
    fn list(&self, path: &Path) -> Result<Vec<String>, VfsError> {
        let prefix = if path == "/" { String::new() } else { format!("{path}/") };
        let mut out: Vec<String> = self.files.borrow().keys()
            .filter(|k| k.starts_with(&prefix))
            .filter_map(|k| {
                k.strip_prefix(&prefix)
                    .map(|r| r.trim_start_matches('/'))
                    .and_then(|r| r.split('/').next())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
            })
            .collect();
        out.sort();
        out.dedup();
        Ok(out)
    }
    fn mkdir(&self, path: &Path) -> Result<(), VfsError> {
        self.dirs.borrow_mut().insert(path.clone());
        Ok(())
    }
    fn remove(&self, path: &Path) -> Result<(), VfsError> {
        if self.files.borrow_mut().remove(path).is_some() { return Ok(()); }
        if self.dirs.borrow_mut().remove(path) { return Ok(()); }
        Err(VfsError::NotFound(path.clone()))
    }
    fn stat(&self, path: &Path) -> Result<FileMeta, VfsError> {
        let files = self.files.borrow();
        if let Some(d) = files.get(path) {
            return Ok(FileMeta { size: d.len() as u64, is_dir: false, is_file: true, readonly: false });
        }
        drop(files);
        if self.dirs.borrow().contains(path) {
            return Ok(FileMeta { size: 0, is_dir: true, is_file: false, readonly: false });
        }
        Err(VfsError::NotFound(path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_table_routes_longest_prefix() {
        let mut v = MountTable::new();
        v.mount("/", Box::new(MemFs::new()));
        v.mount("/usr", Box::new(MemFs::new()));
        assert_eq!(v.len(), 2);
        // write to /usr/local/x should route to /usr fs
        v.write("/usr/local/x", b"hi").unwrap();
        assert_eq!(v.read("/usr/local/x").unwrap(), b"hi");
    }

    #[test]
    fn memfs_read_write_list() {
        let fs = MemFs::new();
        fs.write(&"/a/b.txt".into(), b"data").unwrap();
        assert_eq!(fs.read(&"/a/b.txt".into()).unwrap(), b"data");
        let list = fs.list(&"/".into()).unwrap();
        assert!(list.contains(&"a".to_string()));
    }

    #[test]
    fn normalize_paths() {
        assert_eq!(normalize("/a/b/../c"), "/a/c");
        assert_eq!(normalize("//a//b"), "/a/b");
    }
}
