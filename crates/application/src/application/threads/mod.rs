use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{Builder, JoinHandle},
};

use uuid::Uuid;

enum TaskStatus {
    NotStarted,
    _Started,
    _Finished,
    _Error,
}

type TaskFn = dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'static;

#[allow(unused)]
struct Task {
    task_fn: Box<TaskFn>,
    id: uuid::Uuid,
    status: TaskStatus,
}

type TaskReceiver = Arc<Mutex<Receiver<Task>>>;

#[allow(unused)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    task_sender: Sender<Task>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);
        let (task_sender, task_queue) = channel::<Task>();
        let task_queue: TaskReceiver = Arc::new(Mutex::new(task_queue));
        for id in 0..size {
            workers.push(Worker::new(id, task_queue.clone()));
        }

        Self {
            workers,
            task_sender,
        }
    }

    pub fn spawn(
        &self,
        task_fn: impl FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'static,
    ) -> Uuid {
        let id = Uuid::new_v4();
        self.task_sender
            .send(Task {
                task_fn: Box::new(task_fn),
                id,
                status: TaskStatus::NotStarted,
            })
            .expect("Failed to send message");
        id
    }
}

#[allow(unused)]
struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(
        id: usize,
        task_queue: TaskReceiver,
    ) -> Self {
        let thread = Builder::new()
            .name(format!("Worker {}", &id))
            .spawn(move || loop {
                let task = task_queue
                    .lock()
                    .expect("Failed to obtain lock on the task queue.")
                    .recv()
                    .expect("Failed to get task");
                log::info!("Worker {}: STARTING:  Task {}", id, task.id);
                // TODO: Collect metrics / results here
                let future = (task.task_fn)();
                tokio::spawn(future);

                todo!("This is a custom implementation, use tokio for now.");
            })
            .expect("Failed to spawn thread for worker");
        Self {
            id,
            thread,
        }
    }
}
