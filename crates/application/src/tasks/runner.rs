use std::{
    any::TypeId,
    collections::HashMap,
    pin::Pin,
    sync::mpsc::{channel, Receiver, Sender},
};

use chrono::NaiveDateTime;
use futures::Future;
use serde::Serialize;
use uuid::Uuid;

use crate::application::http::route::ServerContext;

use super::{TaskScheduler, Ticket};

#[derive(Default, Eq, PartialEq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
pub enum TaskStatus {
    #[default]
    NotStarted,
    Started,
    Completed,
    Canceled,
    Failed,
}

impl<T: ToString> From<T> for TaskError {
    fn from(value: T) -> Self {
        Self::Unknown(value.to_string())
    }
}

// TODO: Move this to the DB for persistence. Need to do some better type/ignore mapping in the ORM first.
pub struct TaskRequest {
    id: uuid::Uuid,
    #[allow(unused)]
    create_date: NaiveDateTime,
    // status: TaskStatus,
    type_id: TypeId,
    data: Vec<u8>,
}

pub struct TaskContext {}

impl TaskRequest {
    pub fn new<T: Serialize + 'static>(data: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            create_date: chrono::Utc::now().naive_utc(),
            type_id: TypeId::of::<T>(), // TODO: Not a reliable way to pass this around. Need to enum-ize it, or otherwise use a static representation.
            data: serde_json::to_vec(&data).unwrap(),
        }
    }
    pub fn get_ticket(&self) -> Ticket {
        Ticket {
            id: self.id,
        }
    }
    pub fn ticket(&self) -> Ticket {
        self.get_ticket()
    }
}

pub type TaskResult = Result<(), TaskError>;
// struct Runnable = dyn Fn() -> String;
#[derive(Debug)]
pub enum TaskError {
    TaskNotFound,
    Unknown(String),
}

type Handler<Req, Ctx, Res> =
    Box<dyn Send + 'static + Fn(Req, Ctx) -> Pin<Box<dyn Future<Output = Res>>>>;
type TaskHandler = Handler<TaskRequest, ServerContext, TaskResult>;
// Going to use the same pattern as [IntoRouteHandler<F, Tag, IO>].
// Start thinking of an abstraction approach. Maybe add a <Req, Res> to the TaskHandler type?
pub trait IntoTaskHandler<F, Tag, Req> {
    fn into(self) -> TaskHandler;
}

pub struct TaskExecutor {
    handlers: HashMap<TypeId, TaskHandler>,
    task_queue: Receiver<TaskRequest>,
    task_sender: Sender<TaskRequest>,
}

impl Default for TaskExecutor {
    fn default() -> Self {
        let (task_sender, task_queue) = channel::<TaskRequest>();
        Self {
            handlers: Default::default(),
            task_queue,
            task_sender,
        }
    }
}

macro_rules! generate_trait_impl {
    (R, $($context_id:ident),*) => {
        impl<F, I, $($context_id,)* O, Fut, >
            IntoTaskHandler<F, ($($context_id,)* I, O, Fut), I> for F
        where
            F: Fn(I, $($context_id),*) -> Fut + Send + Copy + 'static ,
            I: FromTaskRequest + Sized + 'static,
            $($context_id: From<ServerContext> + Sized + 'static,)*
            O: IntoTaskResult + Sized + Send + 'static,
            Fut: std::future::Future<Output = O> + 'static + Send,
        {
            fn into(self) -> TaskHandler {
                    Box::new(move |req, ctx| {
                        Box::pin(async move {
                            self(I::from(req), $($context_id::from(ctx.clone())),*)
                                .await
                                .into()
                        })
                    })
            }
        }

        impl<F, I, $($context_id,)* O>
            IntoTaskHandler<F, ($($context_id,)* I, O), I> for F
        where
            F: Fn(I, $($context_id),*) -> O + Send + Copy + 'static ,
            I: FromTaskRequest + Sized + 'static,
            $($context_id: From<ServerContext> + Sized + 'static,)*
            O: IntoTaskResult + Sized + Send + 'static,
        {
            fn into(self) -> TaskHandler {
                    Box::new(move |req, ctx| {
                        Box::pin(async move {
                            self(I::from(req), $($context_id::from(ctx.clone())),*)
                                .into()
                        })
                    })
            }
        }
    };
}

impl<F, Req, O, Fut> IntoTaskHandler<F, ((), Req, O, Fut), Req> for F
where
    F: Fn(Req) -> Fut + Send + Copy + 'static,
    Req: FromTaskRequest + Sized + 'static,
    O: IntoTaskResult + Sized + Send + 'static,
    Fut: std::future::Future<Output = O> + 'static + Send,
{
    fn into(self) -> TaskHandler {
        Box::new(move |req, _ctx: ServerContext| {
            Box::pin(async move { self(Req::from(req)).await.into() })
        })
    }
}
impl<F, Req, O> IntoTaskHandler<F, ((), Req, O), Req> for F
where
    F: Fn(Req) -> O + Send + Copy + 'static,
    Req: FromTaskRequest + Sized + 'static,
    O: IntoTaskResult + Sized + Send + 'static,
{
    fn into(self) -> TaskHandler {
        Box::new(move |req, _ctx: ServerContext| {
            Box::pin(async move { self(Req::from(req)).into() })
        })
    }
}
generate_trait_impl!(R, C1);
// generate_trait_impl!(R, C1, C2);
// generate_trait_impl!(R, C1, C2, C3,);
// generate_trait_impl!(R, C1, C2, C3, C4);
// generate_trait_impl!(R, C1, C2, C3, C4, C5);

/// From / To implementations - required in order to make this generic over foreign types.
/// Either it can't be done without creating custom versions of the From/To traits,
/// or I'm missing something obvious. I spent a long time trying to navigate this
/// with the compiler.
mod from_to_impl {
    use serde::{Deserialize, Serialize};

    use super::{TaskRequest, TaskResult};

    pub trait FromTaskRequest {
        fn from(req: TaskRequest) -> Self;
    }

    impl<T: for<'a> Deserialize<'a>> FromTaskRequest for T {
        fn from(req: TaskRequest) -> Self {
            serde_json::from_slice(&req.data).unwrap()
        }
    }
    pub trait IntoTaskResult {
        fn into(self) -> TaskResult;
    }
    impl<T: Serialize> IntoTaskResult for T {
        fn into(self) -> TaskResult {
            Ok(())
        }
    }
}
pub use from_to_impl::*;

#[derive(Serialize)]
pub(crate) enum Signal {
    #[allow(unused)]
    Kill,
}

impl TaskExecutor {
    pub fn add_handler<F, T, Req>(
        &mut self,
        f: F,
    ) where
        F: IntoTaskHandler<F, T, Req> + Sized + Send + 'static,
        // F: IntoTaskHandler<F, T, O> + Sized + Send + 'static,
        Req: 'static,
    {
        log::debug!("Adding Request Type: {:?}", TypeId::of::<Req>());
        self.handlers.insert(TypeId::of::<Req>(), f.into());
    }

    // pub async fn run_in_new_thread(
    //     self,
    //     context: ServerContext,
    // ) -> JoinHandle<()> {
    //     // std::thread::spawn(move || self.run(context))
    // }

    #[tokio::main(flavor = "current_thread")]
    pub async fn run(
        self,
        context: ServerContext,
    ) {
        while let Ok(task) = self.task_queue.recv() {
            let id = task.id;
            if TypeId::of::<Signal>() == task.type_id {
                // Any "Signal" is treated as kill for time being
                break;
            }
            match self.handle_task(task, context.clone()).await {
                Ok(_) => {
                    log::info!("[TASK {id}] COMPLETED TASK");
                },
                Err(e) => {
                    log::error!("[TASK {id}] Error while processing task: {:?}", e);
                },
            };
        }
    }

    pub fn scheduler(&self) -> TaskScheduler {
        TaskScheduler {
            task_queue: self.task_sender.clone(),
        }
    }

    async fn handle_task(
        &self,
        task: TaskRequest,
        context: ServerContext,
    ) -> TaskResult {
        log::debug!("Retrieving Request Type: {:?}", &task.type_id);
        let handler = self.handlers.get(&task.type_id).ok_or(TaskError::TaskNotFound)?;
        handler(task, context).await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn handles_tasks_in_thread() {
        todo!()
    }
}
