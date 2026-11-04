mod config;
mod error;
mod service;
mod memory;
mod scheduler;
mod interrupt;

use config::Config;
use service::Service;
use memory::MemoryAllocator;
use scheduler::{Scheduler, Priority, Process, ProcessState};
use interrupt::InterruptHandler;
use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    
    let config = Config::default();
    let mut service = Service::new(config);
    
    service.start()?;
    println!("Service started successfully");
    
    // Demonstrate memory management
    let mut allocator = MemoryAllocator::new();
    let ptr = allocator.allocate(4096, 8).map_err(|e| anyhow::anyhow!(e))?;
    println!("Allocated {} bytes", allocator.allocated());
    allocator.deallocate(ptr, 4096, 8);
    println!("Deallocated memory");
    
    // Demonstrate scheduling
    let mut scheduler = Scheduler::new();
    scheduler.add_process(Process {
        id: 1,
        priority: Priority::High,
        state: ProcessState::Ready,
    });
    let scheduled = scheduler.schedule();
    println!("Scheduled process: {:?}", scheduled);
    
    // Demonstrate interrupt handling
    let handler = InterruptHandler::new();
    handler.enable();
    handler.handle(1).map_err(|e| anyhow::anyhow!(e))?;
    println!("Interrupt handled successfully");
    
    service.stop()?;
    println!("Service stopped successfully");
    
    Ok(())
}
