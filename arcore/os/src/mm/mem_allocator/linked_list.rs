
//! # 侵入式链表实现（用于Buddy System内存分配器）
//!
//! 该实现提供了一个侵入式链表，用于在Buddy System内存分配器中管理内存块。
//! 链表的链域和数据域是完全一致的，这使得链表节点能够高效地存储内存块地址。
//!
//! 链表提供的操作包括：
//! - **`push`**：将内存块的地址添加到链表的前端。
//! - **`pop`**：移除并返回链表中第一个内存块的地址。
//! - **`iter`**：提供一个不可变的迭代器，用于遍历链表。
//! - **`iter_mut`**：提供一个可变的迭代器，用于在遍历时修改链表。
//!
//! ## 安全性
//!
//! 该链表使用原始指针来管理链表节点，因此涉及到 `unsafe` 操作。用户必须确保指针指向有效的内存地址，并且正确管理内存，
//! 避免悬空指针或重复释放等未定义行为。
//!
//! ## 使用示例
//! ```
//! let mut list = LinkedList::new();
//! let block = &mut 42 as *mut usize;
//! unsafe { list.push(block); }
//! let popped = list.pop();
//! assert_eq!(popped, Some(block));
//! ```
#![allow(dead_code)]
use core::{fmt, ptr};

/// 侵入式链表结构体
/// 
/// 该结构体通过 `head` 指针指向链表的第一个元素。链表节点直接存储了内存块的地址。
/// 链表操作通过修改该指针来进行节点插入和移除。
#[derive(Copy, Clone)]
pub struct LinkedList {
    head: *mut usize,  // 链表头指针，指向第一个节点
}

unsafe impl Send for LinkedList {}  // 实现 `Send`，表示该结构体可以在线程之间传递

impl LinkedList {
    
    /// 创建一个新的空链表实例
    /// 
    /// # 返回值
    /// 返回一个新的空链表实例，链表头指针为 `null_mut`。
    /// 
    /// # 示例
    /// ```
    /// let list = LinkedList::new();
    /// ```
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),  // 初始化时链表为空
        }
    }
    
    /// 检查链表是否为空
    /// 
    /// # 返回值
    /// 如果链表为空，返回 `true`；否则返回 `false`。
    /// 
    /// # 示例
    /// ```
    /// assert!(list.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.head.is_null()  // 判断头指针是否为空
    }

    /// 将一个内存块的地址添加到链表的前端
    /// 
    /// # 参数
    /// - `item`: 要添加的内存块地址，必须是有效的原始指针。
    /// 
    /// # 安全性
    /// 该操作是 `unsafe`，因为我们直接操作原始指针，用户必须确保提供的指针有效。
    /// 
    /// # 示例
    /// ```
    /// let mut block = &mut 42 as *mut usize;
    /// unsafe { list.push(block); }
    /// ```
    pub unsafe fn push(&mut self, item: *mut usize) {
        unsafe { *item = self.head as usize };  // 将当前链表头指针存储到新节点中
        self.head = item;  // 更新链表头指针为新节点
    }

    /// 从链表中移除并返回第一个内存块的地址
    /// 
    /// # 返回值
    /// 如果链表非空，返回链表头指针指向的内存块地址；否则返回 `None`。
    /// 
    /// # 示例
    /// ```
    /// let addr = list.pop();
    /// assert!(addr.is_some());
    /// ```
    pub fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,  // 链表为空时返回 None
            false => {
                let item = self.head;  // 获取链表头指针
                self.head = unsafe { *item as *mut usize };  // 更新链表头指针为下一个节点
                Some(item)  // 返回移除的节点
            }
        }
    }

    /// 返回一个不可变的迭代器，用于遍历链表中的内存块地址
    /// 
    /// # 返回值
    /// 返回一个不可变迭代器，用于遍历链表中的每个节点。
    /// 
    /// # 示例
    /// ```
    /// for addr in list.iter() {
    ///     println!("{:x}", addr as usize);
    /// }
    /// ```
    pub fn iter(&self) -> Iter {
        Iter {
            curr: self.head,  // 迭代器的当前节点初始化为链表头
            list: self,  // 迭代器持有链表的引用
        }
    }

    /// 返回一个可变的迭代器，用于在遍历时修改链表
    /// 
    /// # 返回值
    /// 返回一个可变迭代器，允许修改链表中的节点。
    /// 
    /// # 示例
    /// ```
    /// for node in list.iter_mut() {
    ///     node.pop();  // 移除当前节点
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,  // prev 初始化为指向链表头指针的指针
            curr: self.head,  // 当前节点初始化为链表头
            list: self,  // 迭代器持有链表的可变引用
        }
    }
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()  // 使用迭代器遍历链表并输出
    }
}

/// 不可变迭代器，用于遍历链表中的节点
pub struct Iter<'a> {
    curr: *mut usize,  // 当前节点指针
    list: &'a LinkedList,  // 持有链表的不可变引用
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut usize;  // 迭代器返回的类型是内存块的地址

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None  // 如果当前节点为空，返回 None
        } else {
            let item = self.curr;  // 获取当前节点
            let next = unsafe { *item as *mut usize };  // 获取下一个节点的指针
            self.curr = next;  // 更新当前节点为下一个节点
            Some(item)  // 返回当前节点
        }
    }
}

/// 链表节点结构体，用于在可变迭代器中操作链表
pub struct ListNode {
    prev: *mut usize,  // 指向前一个节点的指针
    curr: *mut usize,  // 当前节点的指针
}

impl ListNode {
    /// 从链表中移除当前节点并返回其地址
    /// 
    /// # 返回值
    /// 返回当前节点的地址，并移除该节点。
    pub fn pop(self) -> *mut usize {
        unsafe {
            *(self.prev) = *(self.curr);  // 将前一个节点的指针指向当前节点的下一个节点
        }
        self.curr  // 返回当前节点地址
    }

    /// 获取当前节点的内存块地址
    /// 
    /// # 返回值
    /// 返回当前节点的内存块地址。
    pub fn value(&self) -> *mut usize {
        self.curr  // 返回当前节点的地址
    }
}

/// 可变迭代器，用于在遍历链表时修改节点
pub struct IterMut<'a> {
    list: &'a mut LinkedList,  // 持有链表的可变引用
    prev: *mut usize,  // 前一个节点的指针
    curr: *mut usize,  // 当前节点的指针
}

impl<'a> Iterator for IterMut<'a> {
    type Item = ListNode;  // 迭代器返回的是一个 `ListNode`，代表当前节点

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None  // 如果当前节点为空，返回 None
        } else {
            let res = ListNode {
                prev: self.prev,  // 返回当前节点的前一个节点和当前节点
                curr: self.curr,
            };
            self.prev = self.curr;  // 更新前一个节点为当前节点
            self.curr = unsafe { *self.curr as *mut usize };  // 更新当前节点为下一个节点
            Some(res)  // 返回当前节点
        }
    }
}

/// 主函数示例
/// 
/// 测试链表的基本操作，插入节点并验证链表的正确性。
#[allow(unused)]
fn main() {
    let mut a: usize = 1;  // 示例数据
    let mut b: usize = 2;
    let mut c :usize = 3;
    let mut list = LinkedList::new();  // 创建一个空链表

    // 获取变量的原始指针并打印地址
    let addr1 = &mut a  as *mut usize;
    println!("address of A: {:x}", addr1 as usize);
    let addr2 = &mut b  as *mut usize;
    println!("address of B: {:x}", addr2 as usize);
    let addr3 = &mut c as *mut usize;
    println!("address of C: {:x}", addr3 as usize);

    // 将地址压入链表
    let addr_arr = [addr1, addr2, addr3];
    for addr in addr_arr.iter().rev() {
        unsafe { list.push(*addr) };
    }

    // 打印链表内容
    println!("list: {:?}", list);

    // 验证链表内容
    for (i, addr) in list.iter().enumerate() {
        assert_eq!(addr, addr_arr[i]);
    }
}
