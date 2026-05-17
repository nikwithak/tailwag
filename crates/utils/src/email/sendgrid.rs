use reqwest;
use serde::Serialize;

use crate::email::{error::EmailError, EmailAddress, EmailProvider};

pub struct SendGridEmailClient {
    api_key: String,
    sender_email: String,
}

impl EmailProvider for SendGridEmailClient {
    async fn send_email(
        &self,
        to: &EmailAddress,
        subject: &str,
        body_content: &str,
        _reply_to: Option<&EmailAddress>,
    ) -> Result<(), EmailError> {
        let response = reqwest::Client::new()
            // TODO: Set URL as a config / ENV variable.
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("BEARER {}", &self.api_key))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&SendGridEmailRequest::new(
                to.to_string(),
                self.sender_email.to_string(),
                subject.to_string(),
                body_content.to_string(),
            ))?)
            .send()
            .await?;

        response.error_for_status()?;
        Ok(())
    }
}

impl SendGridEmailClient {
    pub fn new(
        api_key: String,
        sender_email: String,
    ) -> Self {
        Self {
            api_key,
            sender_email,
        }
    }

    pub fn from_env() -> Result<Self, EmailError> {
        let ret = Self {
            api_key: std::env::var("SENDGRID_API_KEY")?,
            sender_email: std::env::var("SENDGRID_SENDER_EMAIL")?,
        };
        Ok(ret)
    }
}

#[derive(Serialize)]
struct SendGridEmailRequest {
    personalizations: Vec<SendGridPersonalization>,
    from: SendGridEmailDetail,
    subject: String,
    content: Vec<SendGridContent>,
}

#[derive(Serialize)]
struct SendGridEmailDetail {
    email: String,
}

#[derive(Serialize)]
struct SendGridPersonalization {
    to: Vec<SendGridEmailDetail>,
}

#[derive(Serialize)]
#[allow(unused)]
enum SendGridContentType {
    #[serde(rename = "text/plain")]
    Plaintext,
    #[serde(rename = "text/html")]
    Html,
}

#[derive(Serialize)]
struct SendGridContent {
    #[serde(rename = "type")]
    content_type: SendGridContentType,
    value: String,
}

// Reference SendGrid API for details
impl SendGridEmailRequest {
    fn new(
        to_email: String,
        from_email: String,
        subject: String,
        email_body: String,
    ) -> Self {
        // Quick and dirty - TODO revamp this later
        SendGridEmailRequest {
            personalizations: vec![SendGridPersonalization {
                to: vec![SendGridEmailDetail {
                    email: to_email,
                }],
            }],
            from: SendGridEmailDetail {
                email: from_email,
            },
            subject,
            content: vec![SendGridContent {
                content_type: SendGridContentType::Plaintext,
                value: email_body,
            }],
        }
    }
}
