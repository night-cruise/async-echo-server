use crate::task::{Task, TaskId};
use crate::{Arc, BTreeMap, Context, Future, Poll, Waker};

use crossbeam_channel::{bounded, Receiver, Sender};

pub struct Executor {
    task_queue: Receiver<Arc<Task>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

#[derive(Clone)]
pub struct Spawner {
    task_sender: Sender<Arc<Task>>,
}

impl Executor {
    fn new(task_queue: Receiver<Arc<Task>>) -> Self {
        Self {
            task_queue,
            waker_cache: BTreeMap::new(),
        }
    }

    fn run_ready_task(&mut self) {
        while let Ok(task) = self.task_queue.recv() {
            let waker = self
                .waker_cache
                .entry(task.task_id())
                .or_insert_with(|| Waker::from(task.clone()));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(_) => {
                    self.waker_cache.remove(&task.task_id());
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) {
        self.run_ready_task();
    }
}

impl Spawner {
    fn new(task_sender: Sender<Arc<Task>>) -> Self {
        Self { task_sender }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let task = Task::new(future, self.task_sender.clone());
        self.task_sender
            .send(Arc::new(task))
            .expect("send task failed");
    }
}

pub fn spawner_and_executor() -> (Spawner, Executor) {
    let (task_sender, task_queue) = bounded(10000);
    let spawner = Spawner::new(task_sender);
    let executor = Executor::new(task_queue);
    (spawner, executor)
}
