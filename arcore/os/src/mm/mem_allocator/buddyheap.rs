use super::linked_list;
use core::mem::size_of;
use core::ptr::NonNull;
use core::cmp::{max,min};
use core::alloc::{GlobalAlloc, Layout};
use crate::sync::UPSafeCell;

/// Buddy 系统内存分配器
/// 
/// `Heap` 实现了一个经典的 Buddy 内存分配算法，它通过将内存分为多个大小为 2 的幂次的块来实现内存的高效分配和回收。
/// 该分配器支持内存块的合并和拆分，具有较好的内存碎片管理能力。`Heap` 可以用于低级系统编程，如操作系统或嵌入式系统的内存管理。
/// 
/// 该分配器将空闲内存块按块大小组织在 32 个链表中，每个链表对应一种大小的块。`Heap` 提供了内存分配和释放的基本操作，
/// 并且能够根据需要合并相邻的空闲块（Buddy）以减少内存碎片。
///
pub struct BuddyAllocator {
    pub inner:UPSafeCell<Option<Heap>>,
}
#[allow(unused)]
impl BuddyAllocator {
    pub  const fn empty() -> Self {
        Self {
            inner: unsafe { UPSafeCell::new(None) },
        }
    }
    pub fn init(&self, start: usize, size: usize) {
        let mut  heap=Heap::new();
        unsafe { heap.init(start, size) };
        self.inner.exclusive_access().replace(unsafe { heap });
    }
}

unsafe impl GlobalAlloc for BuddyAllocator{
     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
         self.inner.exclusive_access().as_mut().unwrap().alloc(layout)
         .ok()
         .map_or(core::ptr::null_mut::<u8>(), |allocation| allocation.as_ptr())
     }
     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.exclusive_access().as_mut().unwrap().dealloc(unsafe { NonNull::new_unchecked(ptr) }, layout)
     }
}

pub struct Heap {
    /// 空闲内存块链表，每个链表对应一种大小的块。
    /// `free_list[i]` 存储所有大小为 `2^i` 的空闲块的指针。
    free_list: [linked_list::LinkedList; 32],

    /// 当前用户分配的内存总量（字节数）。
    user: usize,

    /// 当前堆中已分配的内存总量（字节数）。
    allocated: usize,

    /// 堆中总的内存量（字节数）。
    total: usize,
}

impl Heap {
    /// 创建一个新的 `Heap` 实例，初始化空的链表和内存统计信息。
    ///
    /// # 返回值
    /// 返回一个新的 `Heap` 实例，所有空闲块链表为空。
    ///
    /// # 示例
    /// ```
    /// let heap = Heap::new();
    /// ```
    pub const fn new() -> Self {
        Heap {
            free_list: [linked_list::LinkedList::new(); 32],
            user: 0,
            allocated: 0,
            total: 0,
        }
    }

   

    /// 将一块内存区域加入到堆中，并按 Buddy 系统的规则进行管理。
    ///
    /// # 参数
    /// - `start`: 内存区域的起始地址（字节）。
    /// - `end`: 内存区域的结束地址（字节）。
    ///
    /// # 安全性
    /// 该方法需要传入有效的内存地址范围，且只能在不对堆造成冲突的情况下调用。
    ///
    /// # 示例
    /// ```
    /// unsafe { heap.add_to_heap(start, end); }
    /// ```
    pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
        // 对齐内存起始地址，以适应对齐要求。
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end &= !size_of::<usize>() + 1;
        assert!(start <= end);

        let mut total = 0;
        let mut current_start = start;

        while current_start + size_of::<usize>() <= end {
            let lowbit = current_start & (!current_start + 1);
            let size = min(lowbit, prev_power_of_two(end - current_start));
            total += size;

            // 将内存块插入对应大小的链表中。
            unsafe { self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize) };
            current_start += size;
        }

        self.total += total;
    }

    /// 初始化堆内存。
    ///
    /// # 参数
    /// - `start`: 堆的起始地址。
    /// - `size`: 堆的大小。
    ///
    /// # 安全性
    /// 该方法会直接操作堆的内存，因此需要确保提供的内存区域有效且不与其他内存区域冲突。
    ///
    /// # 示例
    /// ```
    /// unsafe { heap.init(start, size); }
    /// ```
    pub unsafe fn init(&mut self, start: usize, size: usize){
        unsafe { self.add_to_heap(start, start + size) };
        
    }

    /// 从堆中分配内存，满足对齐要求并返回分配结果。
    ///
    /// # 参数
    /// - `layout`: 请求的内存布局，包含大小和对齐要求。
    ///
    /// # 返回值
    /// 如果内存分配成功，返回分配的内存指针。如果分配失败，返回 `Err(())`。
    ///
    /// # 示例
    /// ```
    /// let layout = Layout::new::<usize>();
    /// let ptr = heap.alloc(layout);
    /// ```
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );

        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len() {
            // 寻找合适的块并拆分，直到满足请求的内存大小。
            if !self.free_list[i].is_empty() {
                for j in (class + 1..i + 1).rev() {
                    if let Some(block) = self.free_list[j].pop() {
                        
                            unsafe { self.free_list[j - 1]
                                .push((block as usize + (1 << (j - 1))) as *mut usize) };
                            unsafe { self.free_list[j - 1].push(block) };
                        
                    } else {
                        return Err(());  // 找不到足够的内存块时返回错误。
                    }
                }

                let result = NonNull::new(
                    self.free_list[class]
                        .pop()
                        .expect("当前内存块应有足够空间")
                        as *mut u8,
                );
                if let Some(result) = result {
                    self.user += layout.size();
                    self.allocated += size;
                    return Ok(result);  // 返回成功分配的内存指针。
                } else {
                    return Err(());  // 无法分配内存时返回错误。
                }
            }
        }
        Err(())  // 如果没有找到合适的内存块，返回错误。
    }

    /// 释放指定内存块。
    ///
    /// # 参数
    /// - `ptr`: 要释放的内存指针。
    /// - `layout`: 释放内存的布局，包含内存块的大小和对齐要求。
    ///
    /// # 示例
    /// ```
    /// heap.dealloc(ptr, layout);
    /// ```
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;

        unsafe {
            // 将内存块插入到空闲链表中。
            self.free_list[class].push(ptr.as_ptr() as *mut usize);

            // 合并相邻的空闲块（Buddy）。
            let mut current_ptr = ptr.as_ptr() as usize;
            let mut current_class = class;
            while current_class < self.free_list.len() {
                let buddy = current_ptr ^ (1 << current_class);
                let mut flag = false;
                for block in self.free_list[current_class].iter_mut() {
                    if block.value() as usize == buddy {
                        block.pop();
                        flag = true;
                        break;
                    }
                }

                if flag {
                    self.free_list[current_class].pop();
                    current_ptr = min(current_ptr, buddy);
                    current_class += 1;
                    self.free_list[current_class].push(current_ptr as *mut usize);
                } else {
                    break;  // 如果没有合适的 Buddy 块，则停止合并。
                }
            }
        }

        self.user -= layout.size();
        self.allocated -= size;
    }
}

/// 返回小于 `num` 的最大 2 的幂次
/// 
/// 该函数通过获取数字的二进制表示中 1 的最高位置，从而计算出一个小于等于给定数字的最大 2 的幂次。
/// 
/// # 参数
/// - `num`: 输入的数字。
/// 
/// # 返回值
/// 返回小于等于 `num` 的最大 2 的幂次。
/// 
/// # 示例
/// ```
/// let num = 50;
/// let result = prev_power_of_two(num);
/// assert_eq!(result, 32);  // 50 的最大 2 的幂次为 32
/// ```
pub fn prev_power_of_two(num: usize) -> usize {
    1 << (usize::BITS as usize - num.leading_zeros() as usize - 1)
}


