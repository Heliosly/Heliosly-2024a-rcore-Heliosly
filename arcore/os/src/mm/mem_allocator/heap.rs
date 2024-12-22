use crate::config::KERNEL_HEAP_SIZE;
use super::slab::SlabAllocator;


#[global_allocator]
static HEAP_ALLOCATOR: SlabAllocator= SlabAllocator::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
        .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}