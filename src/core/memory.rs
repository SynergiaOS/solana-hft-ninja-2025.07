// 🥷 Memory Management - Zero-Copy, High-Performance Memory Pool
// Optimized memory allocation for sub-microsecond trading operations

use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::ptr::NonNull;
use std::alloc::{Layout, alloc, dealloc};

/// High-performance memory pool for zero-copy operations
pub struct MemoryPool {
    pools: Vec<ObjectPool>,
    total_size: usize,
    used_size: Arc<Mutex<usize>>,
}

impl MemoryPool {
    pub fn new(total_size: usize) -> Result<Self> {
        let mut pools = Vec::new();
        
        // Create pools for different object sizes
        // Optimized for common trading data structures
        pools.push(ObjectPool::new(64, 1000)?);    // Small objects (prices, timestamps)
        pools.push(ObjectPool::new(256, 500)?);    // Medium objects (order book entries)
        pools.push(ObjectPool::new(1024, 100)?);   // Large objects (full order books)
        pools.push(ObjectPool::new(4096, 50)?);    // Extra large objects (market snapshots)
        
        Ok(Self {
            pools,
            total_size,
            used_size: Arc::new(Mutex::new(0)),
        })
    }
    
    /// Allocate memory from the appropriate pool
    pub fn allocate(&self, size: usize) -> Result<PooledMemory> {
        // Find the smallest pool that can accommodate the request
        for pool in &self.pools {
            if pool.object_size >= size {
                if let Some(memory) = pool.allocate() {
                    // Update used size
                    if let Ok(mut used) = self.used_size.lock() {
                        *used += size;
                    }
                    
                    return Ok(PooledMemory {
                        ptr: memory,
                        size,
                        pool_size: pool.object_size,
                        pool_id: pool.id,
                    });
                }
            }
        }
        
        Err(anyhow!("No available memory pool for size {}", size))
    }
    
    /// Deallocate memory back to the pool
    pub fn deallocate(&self, memory: PooledMemory) {
        // Find the appropriate pool and return the memory
        for pool in &self.pools {
            if pool.id == memory.pool_id {
                pool.deallocate(memory.ptr);
                
                // Update used size
                if let Ok(mut used) = self.used_size.lock() {
                    *used = used.saturating_sub(memory.size);
                }
                break;
            }
        }
    }
    
    /// Get memory usage statistics
    pub fn stats(&self) -> MemoryStats {
        let used_size = self.used_size.lock().unwrap_or_else(|_| {
            panic!("Mutex poisoned")
        });
        
        let pool_stats: Vec<PoolStats> = self.pools.iter().map(|pool| pool.stats()).collect();
        
        MemoryStats {
            total_size: self.total_size,
            used_size: *used_size,
            free_size: self.total_size.saturating_sub(*used_size),
            pool_stats,
        }
    }
}

/// Object pool for specific size allocations
pub struct ObjectPool {
    id: usize,
    object_size: usize,
    capacity: usize,
    free_objects: Mutex<VecDeque<NonNull<u8>>>,
    allocated_count: Mutex<usize>,
}

impl ObjectPool {
    fn new(object_size: usize, capacity: usize) -> Result<Self> {
        let mut free_objects = VecDeque::with_capacity(capacity);
        
        // Pre-allocate all objects
        for _ in 0..capacity {
            let layout = Layout::from_size_align(object_size, 8)
                .map_err(|e| anyhow!("Invalid layout: {}", e))?;
            
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                return Err(anyhow!("Failed to allocate memory"));
            }
            
            free_objects.push_back(
                NonNull::new(ptr).ok_or_else(|| anyhow!("Null pointer"))?
            );
        }
        
        Ok(Self {
            id: rand::random(),
            object_size,
            capacity,
            free_objects: Mutex::new(free_objects),
            allocated_count: Mutex::new(0),
        })
    }
    
    fn allocate(&self) -> Option<NonNull<u8>> {
        let mut free_objects = self.free_objects.lock().ok()?;
        let mut allocated_count = self.allocated_count.lock().ok()?;
        
        if let Some(ptr) = free_objects.pop_front() {
            *allocated_count += 1;
            Some(ptr)
        } else {
            None
        }
    }
    
    fn deallocate(&self, ptr: NonNull<u8>) {
        if let (Ok(mut free_objects), Ok(mut allocated_count)) = 
            (self.free_objects.lock(), self.allocated_count.lock()) {
            
            free_objects.push_back(ptr);
            *allocated_count = allocated_count.saturating_sub(1);
        }
    }
    
    fn stats(&self) -> PoolStats {
        let allocated_count = self.allocated_count.lock().unwrap_or_else(|_| {
            panic!("Mutex poisoned")
        });
        
        let free_count = self.free_objects.lock().map(|guard| guard.len()).unwrap_or(0);
        
        PoolStats {
            object_size: self.object_size,
            capacity: self.capacity,
            allocated_count: *allocated_count,
            free_count,
        }
    }
}

impl Drop for ObjectPool {
    fn drop(&mut self) {
        // Deallocate all objects
        if let Ok(mut free_objects) = self.free_objects.lock() {
            let layout = Layout::from_size_align(self.object_size, 8).unwrap();
            
            while let Some(ptr) = free_objects.pop_front() {
                unsafe {
                    dealloc(ptr.as_ptr(), layout);
                }
            }
        }
    }
}

/// Pooled memory handle
pub struct PooledMemory {
    ptr: NonNull<u8>,
    size: usize,
    pool_size: usize,
    pool_id: usize,
}

impl PooledMemory {
    /// Get a raw pointer to the memory
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
    
    /// Get the size of the allocated memory
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get a slice view of the memory
    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.ptr.as_ptr(), self.size)
    }
    
    /// Get a mutable slice view of the memory
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size)
    }
}

unsafe impl Send for PooledMemory {}
unsafe impl Sync for PooledMemory {}

unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}

unsafe impl Send for ObjectPool {}
unsafe impl Sync for ObjectPool {}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_size: usize,
    pub used_size: usize,
    pub free_size: usize,
    pub pool_stats: Vec<PoolStats>,
}

/// Pool-specific statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub object_size: usize,
    pub capacity: usize,
    pub allocated_count: usize,
    pub free_count: usize,
}

impl PoolStats {
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.allocated_count as f64 / self.capacity as f64
        }
    }
}

/// Zero-copy buffer for high-frequency operations
pub struct ZeroCopyBuffer {
    data: Vec<u8>,
    capacity: usize,
}

impl ZeroCopyBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Write data without copying (moves ownership)
    pub fn write_owned(&mut self, data: Vec<u8>) -> Result<()> {
        if data.len() > self.capacity {
            return Err(anyhow!("Data too large for buffer"));
        }
        
        self.data = data;
        Ok(())
    }
    
    /// Get a reference to the data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Clear the buffer without deallocating
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    /// Get buffer statistics
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            capacity: self.capacity,
            used: self.data.len(),
            free: self.capacity.saturating_sub(self.data.len()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferStats {
    pub capacity: usize,
    pub used: usize,
    pub free: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::new(1024 * 1024).unwrap();
        
        // Allocate some memory
        let mem1 = pool.allocate(64).unwrap();
        let mem2 = pool.allocate(256).unwrap();
        
        assert_eq!(mem1.size(), 64);
        assert_eq!(mem2.size(), 256);
        
        // Check stats
        let stats = pool.stats();
        assert_eq!(stats.used_size, 320);
        
        // Deallocate
        pool.deallocate(mem1);
        pool.deallocate(mem2);
        
        let stats = pool.stats();
        assert_eq!(stats.used_size, 0);
    }

    #[test]
    fn test_zero_copy_buffer() {
        let mut buffer = ZeroCopyBuffer::new(1024);
        
        let data = vec![1, 2, 3, 4, 5];
        buffer.write_owned(data).unwrap();
        
        assert_eq!(buffer.data(), &[1, 2, 3, 4, 5]);
        
        let stats = buffer.stats();
        assert_eq!(stats.used, 5);
        assert_eq!(stats.free, 1019);
    }
}
