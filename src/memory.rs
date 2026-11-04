//! Memory management module for kernel operations
//! Follows: Memory Discipline, Memory Safety, Virtual Memory Abstraction

use std::alloc::{self, Layout};
use std::ptr::NonNull;

/// Memory allocator with safety guarantees
pub struct MemoryAllocator {
    allocated: usize,
}

impl MemoryAllocator {
    pub fn new() -> Self {
        MemoryAllocator { allocated: 0 }
    }

    /// Allocate memory with size and alignment
    /// Returns NonNull pointer or error
    pub fn allocate(&mut self, size: usize, align: usize) -> Result<NonNull<u8>, &'static str> {
        let layout = Layout::from_size_align(size, align)
            .map_err(|_| "Invalid layout")?;
        
        let ptr = unsafe { alloc::alloc(layout) };
        
        if ptr.is_null() {
            Err("Allocation failed")
        } else {
            self.allocated += size;
            Ok(NonNull::new(ptr).unwrap())
        }
    }

    /// Deallocate memory safely
    pub fn deallocate(&mut self, ptr: NonNull<u8>, size: usize, align: usize) {
        let layout = Layout::from_size_align(size, align).unwrap();
        unsafe { alloc::dealloc(ptr.as_ptr(), layout) };
        self.allocated -= size;
    }

    /// Get total allocated memory
    pub fn allocated(&self) -> usize {
        self.allocated
    }
}

impl Default for MemoryAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_allocation() {
        let mut allocator = MemoryAllocator::new();
        let ptr = allocator.allocate(1024, 8).unwrap();
        assert_eq!(allocator.allocated(), 1024);
        allocator.deallocate(ptr, 1024, 8);
        assert_eq!(allocator.allocated(), 0);
    }
}
