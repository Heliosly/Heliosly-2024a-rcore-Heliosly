//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.
use crate::{
    config::MEMORY_END,
    mm::{KernelAddr, PhysAddr, PhysPageNum}, sync::UPSafeCell,
    
};
// use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use log::info;

/// manage a frame which has the same lifecycle as the tracker
pub struct FrameTracker {
    ///dd
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    ///Create an empty `FrameTracker`
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
    
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
    fn alloc_contig(&mut self, num: usize) -> Vec<PhysPageNum>;
}
/// an implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    const fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
        // println!("last {} Physical Frames.", self.end - self.current);
        // println!(
        //     "Physical Frames start {:#x}, end {:#x}",
        //     self.current, self.end
        // );
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            println!("cannot alloc!!!!!!! current {:#x}", self.current);
            None
        } else {
            // log::error!("[FrameAllocator::alloc] current {:#x}", self.current);
            self.current += 1;
            Some((self.current - 1)
            .into())
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        // ppn.bytes_array().fill(0);
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
    fn alloc_contig(&mut self, num: usize) -> Vec<PhysPageNum> {
        let mut ret = Vec::with_capacity(num);
        for _ in 0..num {
            if self.current == self.end {
                println!("cannot alloc!!!!!!! current {:#x}", self.current);
                panic!()
            } else {
                self.current += 1;
                ret.push((self.current - 1).into());
            }
        }
        ret
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

pub static FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
   unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(KernelAddr::from(ekernel as usize)).ceil(),
        PhysAddr::from(KernelAddr::from(MEMORY_END)).floor(),
    );
    info!(
        "frame allocator init finshed, start {:#x}, end {:#x}",
        ekernel as usize, MEMORY_END
    );
}
/// allocate contiguous frames
pub fn frame_alloc_contig(num: usize) -> Vec<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc_contig(num)
        .iter()
        .map(|p| FrameTracker::new(*p))
        .collect()
}
/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.exclusive_access().alloc().map(FrameTracker::new)
}

/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    info!("frame_allocator_test start...");
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    info!("frame_allocator_test passed!");
}
