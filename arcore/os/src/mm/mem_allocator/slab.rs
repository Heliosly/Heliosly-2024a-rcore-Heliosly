
use core::{alloc::{AllocError, Layout}, ptr::NonNull};
use super::{linked_list::LinkedList,buddyheap::Heap};
use crate::sync::UPSafeCell;
use core::alloc::GlobalAlloc;
pub struct SlabAllocator {
   pub inner:UPSafeCell<Option<Slabheap>>,
}

impl SlabAllocator {
    pub const fn empty() -> Self {
        Self {
            inner: unsafe { UPSafeCell::new(None) },
        }
    }
   pub fn init(&self,start:usize,size:usize) -> &Self {
       self.inner.exclusive_access().replace(Slabheap::new());
       self.inner.exclusive_access().as_mut().unwrap().init(start,size);
       self
   }
   
}
unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.exclusive_access().as_mut().unwrap().alloc(layout).unwrap()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.exclusive_access().as_mut().unwrap().dealloc(ptr, layout);
    }
}
enum BlockSize {
    Slab64B,
    Slab128B,
    Slab256B,
    Slab512B,
    Slab1024B,
    Slab2048B,
    Slab4096B,
    BuddyHeap,
}
pub struct Slabheap{
    slab64:Slab<64,64>,
    slab128:Slab<128,64>,
    slab256:Slab<256,64>,
    slab512:Slab<512,64>,
    slab1024:Slab<1024,64>,
    slab2048:Slab<2048,64>,
    slab4096:Slab<4096,64>,
    buddy_heap:Heap,
    used:usize,
    allocated:usize,
}
trait FindSuitableSlab {
    fn find_suitable_slab(&self) -> BlockSize;
}
impl FindSuitableSlab for Layout {
    fn find_suitable_slab(&self) -> BlockSize {
        match self.size() {
            0..=64 => BlockSize::Slab64B,
            65..=128 => BlockSize::Slab128B,
            129..=256 => BlockSize::Slab256B,
            257..=512 => BlockSize::Slab512B,
            513..=1024 => BlockSize::Slab1024B,
            1025..=2048 => BlockSize::Slab2048B,
            2049..=4096 => BlockSize::Slab4096B,
            _ => BlockSize::BuddyHeap,
        }
    

    }
}

impl Slabheap {
    pub  fn new()->Slabheap{
        Slabheap{
            slab64:unsafe { Slab::<64,64>::new(0,0) },
            slab128:unsafe { Slab::<128,64>::new(0,0) },
            slab256:unsafe { Slab::<256,64>::new(0,0) },
            slab512:unsafe { Slab::<512,64>::new(0,0) },
            slab1024:unsafe { Slab::<1024,64>::new(0,0) },
            slab2048:unsafe { Slab::<2048,64>::new(0,0) },
            slab4096:unsafe { Slab::<4096,64>::new(0,0) },
            buddy_heap:Heap::new(),
            used:0,
            allocated:0,
        }
    }

    pub fn init(&mut self,start:usize, size:usize){
        unsafe { self.buddy_heap.init(start, size) };


    }
    fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocError> {
        
        self.used+=layout.size();
        
        let ptr:*mut u8=match layout.find_suitable_slab() {
           BlockSize::Slab64B =>{ self.allocated+=64;self.slab64.allocate(layout,&mut self.buddy_heap).unwrap() as *mut u8 }
           BlockSize::Slab128B =>{ self.allocated+=128;self.slab128.allocate(layout,&mut self.buddy_heap).unwrap() as *mut u8 }
           BlockSize::Slab256B =>{ self.allocated+=256;self.slab256.allocate(layout,&mut self.buddy_heap).unwrap() as *mut u8 }
           BlockSize::Slab512B =>{ self.allocated+=512;self.slab512.allocate(layout,&mut self.buddy_heap).unwrap() as *mut u8 }
           BlockSize::Slab1024B =>{ self.allocated+=1024;self.slab1024.allocate(layout,&mut self.buddy_heap).unwrap() as *mut u8 }
           BlockSize::Slab2048B =>{ self.allocated+=2048;self.slab2048.allocate(layout,&mut self.buddy_heap).unwrap() as *mut u8 }
           BlockSize::Slab4096B =>{ self.allocated+=4096;self.slab4096.allocate(layout,&mut self.buddy_heap).unwrap() as *mut u8 }
           _=> {self.allocated+=layout.size();self.buddy_heap.alloc(layout)
           .unwrap().as_ptr()}
        };
        Ok(ptr)
    }
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.used-=layout.size();
        match layout.find_suitable_slab() {
           BlockSize::Slab64B => {self.allocated-=64;self.slab64.deallocate(ptr as usize)},
           BlockSize::Slab128B =>{ self.allocated-=128;self.slab128.deallocate(ptr as usize)},
           BlockSize::Slab256B =>{ self.allocated-=256;self.slab256.deallocate(ptr as usize)},
           BlockSize::Slab512B =>{ self.allocated-=512;self.slab512.deallocate(ptr as usize)},
           BlockSize::Slab1024B =>{ self.allocated-=1024;self.slab1024.deallocate(ptr as usize)},
           BlockSize::Slab2048B =>{ self.allocated-=2048;self.slab2048.deallocate(ptr as usize)},
           BlockSize::Slab4096B =>{ self.allocated-=4096;self.slab4096.deallocate(ptr as usize)},
           _=> {self.allocated-=layout.size();self.buddy_heap.dealloc(unsafe { NonNull::new_unchecked(ptr) }, layout)},
        }
    }
    
}
pub struct Slab<const BLK_SIZE: usize, const SET_SIZE: usize> {
    free_block_list: FreeBlockList<BLK_SIZE>,
    total_blocks: usize,
}
impl<const BLK_SIZE: usize, const SET_SIZE: usize> Slab<BLK_SIZE,SET_SIZE> {
    pub unsafe fn new(start_addr: usize, slab_size: usize) -> Slab<BLK_SIZE,SET_SIZE> {
        let num_of_blocks = slab_size / BLK_SIZE;
        Slab {
            free_block_list: unsafe { FreeBlockList::new(start_addr, BLK_SIZE, num_of_blocks) },
            total_blocks: num_of_blocks,
        }
    }


    pub unsafe fn grow(&mut self, start_addr: usize, slab_size: usize) {
        let num_of_blocks = slab_size / BLK_SIZE;
        self.total_blocks += num_of_blocks;
        let mut block_list = unsafe { FreeBlockList::<BLK_SIZE>::new(start_addr, BLK_SIZE, num_of_blocks) };
        while let Some(block) = block_list.pop() {
            self.free_block_list.push(block);
        }
    }

    pub fn allocate(
        &mut self,
        _layout: Layout,
        buddy: &mut Heap,
    ) -> Result<usize, AllocError> {
        match self.free_block_list.pop() {
            Some(block) => Ok(block as usize),
            None => {
                let layout =
                    unsafe { Layout::from_size_align_unchecked(SET_SIZE * BLK_SIZE, 4096) };
                if let Ok(ptr) = buddy.alloc(layout) {
                    unsafe {
                        self.grow(ptr.as_ptr() as usize, SET_SIZE * BLK_SIZE);
                    }
                    Ok(self.free_block_list.pop().unwrap() as usize)
                } else {
                    Err(AllocError)
                }
            }
        }
    }

    pub fn deallocate(&mut self, ptr: usize) {
        let ptr = ptr as *mut usize;
        unsafe {
            self.free_block_list.push(&mut *ptr);
        }
    }
}

struct FreeBlockList<const BLK_SIZE: usize> {
    len: usize,
    list:LinkedList,
}

impl<const BLK_SIZE: usize> FreeBlockList<BLK_SIZE> {
    unsafe fn new(
        start_addr: usize,
        block_size: usize,
        num_of_blocks: usize,
    ) -> FreeBlockList<BLK_SIZE> {
        let mut new_list = FreeBlockList::new_empty();
        for i in (0..num_of_blocks).rev() {
            let new_block = (start_addr + i * block_size) as *mut usize;
            new_list.push(unsafe { &mut *new_block });
        }
        new_list
    }

    fn new_empty() -> FreeBlockList<BLK_SIZE> {
        FreeBlockList { len: 0, list: LinkedList::new() }
    }

   

    fn pop(&mut self) -> Option<*mut usize> {
        self.list.pop()
    }

    fn push(&mut self, free_block:      *mut usize) {
        self.len += 1;
        unsafe { self.list.push(free_block) };
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}
