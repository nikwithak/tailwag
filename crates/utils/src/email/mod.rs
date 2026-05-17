use std::collections::HashMap;

#[cfg(feature = "smtp2go")]
mod smtp2go;
#[cfg(feature = "sendgrid")]
use crate::email::sendgrid::SendGridEmailClient;
#[cfg(feature = "smtp2go")]
use crate::email::smtp2go::Smtp2GoEmailClient;
use crate::{email::error::EmailError, templates};

pub mod error;
#[cfg(feature = "sendgrid")]
mod sendgrid;

pub type EmailAddress = str;

pub struct EmailTemplate {
    subject: String,
    body_template: String,
}
type TemplateId = String;
pub type Templates = HashMap<TemplateId, EmailTemplate>;

pub trait EmailProvider {
    fn send_email(
        &self,
        to: &EmailAddress,
        subject: &str,
        body_content: &str,
        reply_to: Option<&EmailAddress>,
    ) -> impl std::future::Future<Output = Result<(), EmailError>> + Send;
}

pub struct EmailClient<T: EmailProvider> {
    provider: T,
    templates: Templates,
}

// ClientProviders
#[cfg(feature = "sendgrid")]
impl EmailClient<SendGridEmailClient> {
    pub fn new_sendgrid(
        api_key: String,
        sender_email: String,
        templates: Templates,
    ) -> Self {
        let provider = SendGridEmailClient::new(api_key, sender_email);
        Self::new(provider, templates)
    }
    pub fn from_env() -> Result<Self, EmailError> {
        let provider = SendGridEmailClient::from_env()?;
        Ok(Self::new(provider, Default::default()))
    }
}
#[cfg(feature = "smtp2go")]
impl EmailClient<Smtp2GoEmailClient> {
    pub fn new_smtp2go(
        api_key: String,
        sender_email: String,
        templates: Templates,
    ) -> Self {
        let provider = Smtp2GoEmailClient::new(api_key, sender_email);
        Self::new(provider, templates)
    }
    pub fn smtp2go_from_env() -> Result<Self, EmailError> {
        let provider = Smtp2GoEmailClient::from_env()?;
        Ok(Self::new(provider, Default::default()))
    }
}

impl<T: EmailProvider> EmailProvider for EmailClient<T> {
    fn send_email(
        &self,
        to: &EmailAddress,
        subject: &str,
        body_content: &str,
        reply_to: Option<&EmailAddress>,
    ) -> impl std::future::Future<Output = Result<(), EmailError>> + Send {
        self.provider.send_email(to, subject, body_content, reply_to)
    }
}

impl<T: EmailProvider> EmailClient<T> {
    pub fn new(
        provider: T,
        templates: Templates,
    ) -> Self {
        Self {
            provider,
            templates,
        }
    }
    pub async fn send_template(
        &self,
        to: &str,
        template_id: &TemplateId,
        template_params: &HashMap<String, String>,
        reply_to: Option<&EmailAddress>,
    ) -> Result<(), EmailError> {
        let template = self
            .templates
            .get(template_id)
            .ok_or(EmailError::TemplateNotFound(template_id.clone()))?;
        let filled_template = templates::fill_template(&template.body_template, template_params)?;
        self.send_email(to, &template.subject, &filled_template, reply_to).await
    }
}
