
#![allow(dead_code)]    
extern crate alloc;
use core::{ future::Future, panic};

use spin::Mutex;
use async_task::{Runnable, ScheduleInfo, Task, WithInfo};
use alloc::collections::VecDeque;
use super::TASK_QUEUE;

///exu
pub struct Executor ;

impl Executor {
    
    /// 创建一个异步任务，并将其添加到任务队列中
    pub fn spawn<F>(future: F) -> (Runnable, Task<F::Output>)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    // 创建一个调度函数，用于将任务添加到任务队列中
    let schedule = move |runnable: Runnable, info: ScheduleInfo| {
       // println!("push {:?}",runnable);
        TASK_QUEUE.push(runnable);
        if info.woken_while_running {
         panic!("woken_while_running");
        }  
        
    };
    // 使用async_task库创建一个异步任务，并将其添加到任务队列中
    async_task::spawn(future, WithInfo(schedule))
}

}

/// 定义一个任务队列结构体
pub struct TaskQueue{
    // 使用互斥锁保护任务队列
    queue:Mutex<Option<VecDeque<Runnable>>>,
}

impl TaskQueue{
    /// 创建一个任务队列
    pub const fn new() -> Self {
        Self {
            queue: Mutex::new(None),
        }
    }
    /// 初始化任务队列
    pub fn init(&self) {
        self.queue.lock()
        
        .replace(VecDeque::new());
    }
    
    /// 将任务添加到任务队列的末尾
    pub fn push(&self, runnable: Runnable) {
       // println!("push lock before");
        let mut lock = self.queue.lock();
        //println!("push before, queue len ");
        lock.as_mut().unwrap().push_back(runnable);
        // self.queue.lock().as_mut().unwrap().push_back(runnable);
        // log::error!("push after");
    }
    /// 从任务队列的头部取出一个任务
    pub fn pop(&self) -> Option<Runnable> {
        trace!("kernel:tq pop");
        self.queue.lock().as_mut().unwrap().pop_front()
    }
    /// 将任务添加到任务队列的头部
    pub fn push_preempt(&self, runnable: Runnable) {
        self.queue.lock().as_mut().unwrap().push_front(runnable);
    }
}



/* pub fn spawn_kernel_thread<F: Future<Output = ()> + Send + 'static>(kernel_thread: F) {
    let future = KernelTaskFuture::new(kernel_thread);
    let (runnable, task) = Executor::spawn(future);
    runnable.schedule();
    task.detach();
} */