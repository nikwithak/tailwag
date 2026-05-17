use reqwest;
use serde::Serialize;

use crate::email::{error::EmailError, EmailAddress, EmailProvider};

pub struct Smtp2GoEmailClient {
    api_key: String,
    sender_email: String,
}

impl EmailProvider for Smtp2GoEmailClient {
    async fn send_email(
        &self,
        to: &EmailAddress,
        subject: &str,
        body_content: &str,
        reply_to: Option<&EmailAddress>,
    ) -> Result<(), EmailError> {
        let mut request = Smtp2GoEmailRequest::new(
            to.to_string(),
            self.sender_email.to_string(),
            subject.to_string(),
            body_content.to_string(),
        );
        if let Some(reply_to) = reply_to {
            request = request.with_header("Reply-To", reply_to);
        }
        let response = reqwest::Client::new()
            // TODO: Set URL as a config / ENV variable.
            .post("https://api.smtp2go.com/v3/email/send")
            .header("X-Smtp2go-Api-Key", self.api_key.to_string())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&request)?)
            .send()
            .await?;

        response.error_for_status()?;
        Ok(())
    }
}

impl Smtp2GoEmailClient {
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
            api_key: std::env::var("SMTP2GO_API_KEY")?,
            sender_email: std::env::var("SMTP2GO_SENDER_EMAIL")?,
        };
        Ok(ret)
    }
}

#[derive(Serialize)]
struct Smtp2GoEmailRequest {
    sender: String,
    to: Vec<String>,
    subject: String,
    text_body: String,
    custom_headers: Vec<Smtp2GoCustomHeader>,
}

// Reference Smtp2Go API for details
impl Smtp2GoEmailRequest {
    fn new(
        to_email: String,
        from_email: String,
        subject: String,
        email_body: String,
    ) -> Self {
        // Quick and dirty - TODO revamp this later
        Smtp2GoEmailRequest {
            sender: from_email,
            to: vec![to_email],
            subject,
            text_body: email_body,
            custom_headers: Default::default(),
        }
    }
    fn with_header(
        mut self,
        header_name: impl ToString,
        value: impl ToString,
    ) -> Self {
        self.custom_headers.push(Smtp2GoCustomHeader {
            header: header_name.to_string(),
            value: value.to_string(),
        });
        self
    }
}

#[derive(Serialize)]
struct Smtp2GoCustomHeader {
    header: String,
    value: String,
}
