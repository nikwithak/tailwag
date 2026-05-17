pub mod runner;
use std::sync::mpsc::Sender;

use serde::Serialize;

use crate::application::{
    http::route::{RequestContext, ServerContext},
    ApplicationError,
};

use self::runner::{TaskError, TaskRequest};

/// A manager for asynchronous tasks (worker tasks queued for processing,
/// and returned a result through a callback)
///
/// * A Task may need resources from the server, so we need a way to inject
/// ServerResources / Data / Providers during processing.
///
/// * The task runner will be responible for establishing a worker environment,
/// and running the task inside that environment. e.g. waiting for a worker thread.
///
pub trait Task<T> {
    type Input: Serialize;
    type Output: Serialize;

    fn run(input: Self::Input) -> Self::Output;
}

#[allow(unused)]
struct Worker<T> {
    task: T,
}

#[derive(Default)]
// TODO: Shouldn't be able to default - only new (to generate new ID)
pub struct Ticket {
    id: uuid::Uuid,
}

impl Ticket {
    #[allow(unused)]
    fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
        }
    }
    pub fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

#[allow(unused)]
enum ScheduleError {
    UnknownTask,
    BadRequest,
    Unknown,
}

#[derive(Clone)]
pub struct TaskScheduler {
    task_queue: Sender<TaskRequest>,
    // TODO: Enable DB (or other) storage for TaskRequest, in case of fatal errors (so we don't lose tasks)
    // Things to add:
    //  - Access to DB / storage
    //  - Locking mechanism (NTH)
    //  - Reload failed/pending tasks on startup
    //  - TaskRequest -> helper functions to auto-add
    //  - Status updates on success / failure / other
    // tasks_db: Option<PostgresDataProvider<TaskRequest>>,
}
impl TaskScheduler {
    pub fn enqueue<T: Serialize + 'static>(
        &mut self,
        request_data: T,
    ) -> Result<Ticket, TaskError> {
        let task_request = TaskRequest::new(request_data);
        let ticket = task_request.get_ticket();
        self.task_queue.send(task_request)?;
        log::debug!("[TICKET {}] Sent task to handler", &ticket.id);
        Ok(ticket)
    } // TODO: Return a handle to the job ID
}

// impl

impl From<&RequestContext> for TaskScheduler {
    fn from(ctx: &RequestContext) -> Self {
        ctx.server_context().server_data.get::<Self>().unwrap().clone()
    }
}

impl TryFrom<ServerContext> for TaskScheduler {
    type Error = ApplicationError;
    fn try_from(ctx: ServerContext) -> Result<Self, Self::Error> {
        ctx.server_data
            .get::<Self>()
            .ok_or(ApplicationError::ServerResourceNotFound)
            .cloned()
    }
}
