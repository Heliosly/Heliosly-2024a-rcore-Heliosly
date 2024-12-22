use exu::TaskQueue;



///yibudiaodu
pub mod exu;
///shedule
pub mod shed;
///waker
pub mod waker;
///s
pub fn initexecutor(){
    trace!(
        "initexecutor",
    );
    TASK_QUEUE.init();
    
}

///run loop
pub fn run_until_idle() -> usize {
    
    let mut n = 0;
        while let Some(task) = TASK_QUEUE.pop() {
            info!("fetch a task,runable:{:?}",task);
            task.run();
            n += 1;
        } 
    n
}

static TASK_QUEUE: TaskQueue = TaskQueue::new();


/* use id::TASKID_ALLOCATOR;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(usize);
impl TaskId {
    fn new() -> Self {
        TaskId(TASKID_ALLOCATOR.exclusive_access().alloc())
    }
    
}

pub struct Task {
    id: TaskId, // new
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(), // new
            future: Box::pin(future),
        }
    }
} */

