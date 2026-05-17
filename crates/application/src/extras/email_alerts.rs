use serde::{Deserialize, Serialize};
use tailwag_utils::email::EmailProvider;

use crate::{
    application::WebServiceBuilder,
    tasks::{runner::TaskError, TaskScheduler, Ticket},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SendEmailEvent {
    pub subject: String,
    pub body: String,
    pub recipient: String,
    pub reply_to_address: Option<String>,
}

pub async fn send_email(event: SendEmailEvent) {
    let SendEmailEvent {
        subject,
        body,
        recipient,
        reply_to_address,
    } = event;
    let provider = tailwag_utils::email::EmailClient::smtp2go_from_env().unwrap();
    let client = tailwag_utils::email::EmailClient::new(provider, Default::default());
    client
        .send_email(&recipient, &subject, &body, reply_to_address.as_deref())
        .await
        .unwrap();
}

trait Locked {}
#[allow(private_bounds)]
pub trait WithEmailQueueTask
where
    Self: Locked,
{
    fn with_email_queue_task(self) -> Self;
}
impl Locked for WebServiceBuilder {}
impl WithEmailQueueTask for WebServiceBuilder {
    fn with_email_queue_task(self) -> Self {
        self.with_task(send_email)
    }
}

#[allow(private_bounds)]
pub trait SendEmail
where
    Self: Locked,
{
    fn send_email(
        &mut self,
        subject: impl ToString,
        body: impl ToString,
        recipient: impl ToString,
        reply_to_address: Option<String>,
    ) -> Result<Ticket, TaskError>;
}
impl Locked for TaskScheduler {}
impl SendEmail for TaskScheduler {
    fn send_email(
        &mut self,
        subject: impl ToString,
        body: impl ToString,
        recipient: impl ToString,
        reply_to_address: Option<String>,
    ) -> Result<Ticket, TaskError> {
        self.enqueue(SendEmailEvent {
            subject: subject.to_string(),
            body: body.to_string(),
            recipient: recipient.to_string(),
            reply_to_address: reply_to_address.map(|s| s.to_string()),
        })
    }
}
