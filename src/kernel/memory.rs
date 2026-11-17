//! Virtual memory: address spaces, region mapping, protection.

use std::collections::BTreeMap;

/// A virtual address.
pub type VirtAddr = u64;
/// A physical address (opaque handle in this model).
pub type PhysAddr = u64;

/// Memory protection flags (bitflags-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prot {
    pub read: bool,
    pub write: bool,
    pub exec: bool,
    pub user: bool,
}

impl Prot {
    pub const NONE: Prot = Prot { read: false, write: false, exec: false, user: false };
    pub const R: Prot = Prot { read: true, write: false, exec: false, user: false };
    pub const RW: Prot = Prot { read: true, write: true, exec: false, user: false };
    pub const RWX: Prot = Prot { read: true, write: true, exec: true, user: false };
    pub const USER_RWX: Prot = Prot { read: true, write: true, exec: true, user: true };
}

/// A mapped memory region.
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub vaddr: VirtAddr,
    pub size: u64,
    pub prot: Prot,
    /// Backing physical frame ids (one per page).
    pub frames: Vec<PhysAddr>,
}

impl MemoryRegion {
    pub fn end(&self) -> VirtAddr { self.vaddr.saturating_add(self.size) }
    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr >= self.vaddr && addr < self.end()
    }
}

/// Errors from memory operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemError {
    /// Region overlaps an existing mapping.
    Overlap(VirtAddr),
    /// No region covers the address.
    Unmapped(VirtAddr),
    /// Requested size is zero.
    ZeroSize,
    /// Protection would allow user exec+w (W^X violation).
    WxViolation,
}

impl std::fmt::Display for MemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overlap(a) => write!(f, "mapping overlaps at {a:#x}"),
            Self::Unmapped(a) => write!(f, "unmapped address {a:#x}"),
            Self::ZeroSize => write!(f, "zero-size mapping"),
            Self::WxViolation => write!(f, "W^X violation"),
        }
    }
}
impl std::error::Error for MemError {}

const PAGE: u64 = 4096;

/// A process address space.
#[derive(Debug, Default)]
pub struct AddressSpace {
    regions: BTreeMap<VirtAddr, MemoryRegion>,
    next_frame: PhysAddr,
}

impl AddressSpace {
    pub fn new() -> Self { Self::default() }

    /// Map `size` bytes at `vaddr` (page-aligned) with `prot`.
    /// Allocates backing frames. Fails on overlap or W^X violation.
    pub fn map(&mut self, vaddr: VirtAddr, size: u64, prot: Prot) -> Result<(), MemError> {
        if size == 0 { return Err(MemError::ZeroSize); }
        if prot.write && prot.exec && prot.user {
            return Err(MemError::WxViolation);
        }
        let pages = size.div_ceil(PAGE);
        // check overlap against existing regions
        for r in self.regions.values() {
            if vaddr < r.end() && vaddr + pages * PAGE > r.vaddr {
                return Err(MemError::Overlap(vaddr));
            }
        }
        let mut frames = Vec::with_capacity(pages as usize);
        for _ in 0..pages {
            self.next_frame += 1;
            frames.push(self.next_frame);
        }
        self.regions.insert(vaddr, MemoryRegion { vaddr, size: pages * PAGE, prot, frames });
        Ok(())
    }

    /// Unmap the region containing `vaddr`.
    pub fn unmap(&mut self, vaddr: VirtAddr) -> Result<(), MemError> {
        let key = self.regions.range(..=vaddr).next_back()
            .map(|(&k, _)| k)
            .ok_or(MemError::Unmapped(vaddr))?;
        let r = self.regions.remove(&key).ok_or(MemError::Unmapped(vaddr))?;
        if !r.contains(vaddr) { return Err(MemError::Unmapped(vaddr)); }
        Ok(())
    }

    /// Change protection on the region containing `vaddr`.
    pub fn protect(&mut self, vaddr: VirtAddr, prot: Prot) -> Result<(), MemError> {
        let key = self.regions.range(..=vaddr).next_back()
            .map(|(&k, _)| k)
            .ok_or(MemError::Unmapped(vaddr))?;
        let r = self.regions.get_mut(&key).ok_or(MemError::Unmapped(vaddr))?;
        if !r.contains(vaddr) { return Err(MemError::Unmapped(vaddr)); }
        if prot.write && prot.exec && prot.user { return Err(MemError::WxViolation); }
        r.prot = prot;
        Ok(())
    }

    /// Look up the region containing `addr`.
    pub fn lookup(&self, addr: VirtAddr) -> Option<&MemoryRegion> {
        self.regions.range(..=addr).next_back().and_then(|(_, r)| {
            if r.contains(addr) { Some(r) } else { None }
        })
    }

    /// Translate a virtual address to its backing frame offset.
    pub fn translate(&self, addr: VirtAddr) -> Option<(PhysAddr, u64)> {
        let r = self.lookup(addr)?;
        let off = addr - r.vaddr;
        let idx = (off / PAGE) as usize;
        let frame = *r.frames.get(idx)?;
        Some((frame * PAGE + (off % PAGE), off))
    }

    /// Total mapped bytes.
    pub fn mapped_bytes(&self) -> u64 {
        self.regions.values().map(|r| r.size).sum()
    }

    /// Iterate regions.
    pub fn regions(&self) -> impl Iterator<Item = &MemoryRegion> { self.regions.values() }
}
