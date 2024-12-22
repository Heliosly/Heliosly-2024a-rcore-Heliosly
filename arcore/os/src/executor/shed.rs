use core::future::Future;
//use alloc::boxed::Box;

use alloc::sync::Arc;
use crate::task::taskloop;
use crate::{executor, task::TaskControlBlock};
///spwan
pub fn spawn_user_thread(tcb: Arc<TaskControlBlock>) {
    // let future = schedule::OutermostFuture::new(thread.clone(), async {});
    let (runnable, task) =executor::exu::Executor::spawn( taskloop(tcb));
    runnable.schedule();
    task.detach();
}

/// Spawn a new kernel thread(used for doing some kernel init work or timed tasks)
pub fn spawn_thread<F: Future<Output = ()> + Send + 'static>(future: F) {

    let (runnable, task) = executor::exu::Executor::spawn(future);
    runnable.schedule();
    task.detach();
}       


