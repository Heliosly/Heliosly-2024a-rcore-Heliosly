//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.
use alloc::vec::Vec;
mod address;
mod frame_allocator;
mod mem_allocator;
mod memory_set;
mod page_table;
use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum,KernelAddr};
pub use frame_allocator::{frame_alloc_contig,frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{kernel_token, MapPermission, MemorySet, KERNEL_SPACE};
use page_table::PTEFlags;
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, PageTable,
    PageTableEntry, UserBuffer, UserBufferIterator,
};



/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    trace!(
        "kernel: init memory management"

    );
    mem_allocator::heap::init_heap();
    trace!(
        "kernel: init heapallocator yes"

    );
    frame_allocator::init_frame_allocator();
    trace!(
        "kernel: init heap frameallocator yes"

    );
    KERNEL_SPACE.exclusive_access().activate();
}

    
///test
/* pub fn heap_test2(){
    let time1 = get_time_ms();

    let block_size = 400;
    let block_count=1000;
    for _i in 0..100{
        
        let mut vecs = Vec::with_capacity(block_count);
        for _ in 0..block_count {
            let mut vec = Vec::with_capacity(block_size);
            for i in 0..block_size {
                vec.push(i);
            }
            vecs.push(vec);
        }
    
        for mut vec in vecs {
            for _ in 0..block_size {
                vec.pop();
            }
        }
    }
    
    
    let time2= get_time_ms();
    println!("Slab time: {},block size: {},block count: {}",time2-time1,block_size,block_count);
    panic!("time: {}",time2-time1);
} */

/* #[allow(unused)]
/// test page table and frame allocator
pub fn heap_test2() {
    
    
    println!("Running bumb tests...");

    let mut pool = Vec::new();

    for i in 0.. {
        println!("Indicator: {}", i);
        let mut items = alloc_pass(i);
        free_pass(&mut items, i as u8);
        
        pool.append(&mut items);
        assert_eq!(items.len(), 0);
    }

    println!("Bumb tests run OK!");
}



fn alloc_pass(delta: usize) -> Vec<Vec<u8>> {
    
    let mut items = Vec::new();
    let mut base = 32;
    loop {
       
        let c = (delta % 256) as u8;
        let a = vec![c; base+delta];
        items.push(a);
        if base >= 256*1024 {
            break;
        }
        base *= 2;
    }
    items
}

fn free_pass(items: &mut Vec<Vec<u8>>, delta: u8) {
    let total = items.len();
    for j in (0..total).rev() {
        if j % 2 == 0 {
            let ret = items.remove(j);
            assert_eq!(delta, ret[0]);
            assert_eq!(delta, ret[ret.len()-1]);
        }
    }
} */
/// The memory management framework
pub fn heap_test1() {
    use alloc::boxed::Box;
    trace!(
        "kernel: heap_task",
      
    );
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
