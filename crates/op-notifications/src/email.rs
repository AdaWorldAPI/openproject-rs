//! Email Delivery
//!
//! Mirrors: app/mailers/*.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::notification::{Notification, NotificationType};

/// Email errors
#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("SMTP error: {0}")]
    SmtpError(String),
    #[error("Rate limited")]
    RateLimited,
}

pub type EmailResult<T> = Result<T, EmailError>;

/// Email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    /// Message ID
    pub id: String,
    /// Sender address
    pub from: EmailAddress,
    /// Recipient addresses
    pub to: Vec<EmailAddress>,
    /// CC addresses
    pub cc: Vec<EmailAddress>,
    /// BCC addresses
    pub bcc: Vec<EmailAddress>,
    /// Reply-to address
    pub reply_to: Option<EmailAddress>,
    /// Subject line
    pub subject: String,
    /// Plain text body
    pub text_body: String,
    /// HTML body
    pub html_body: Option<String>,
    /// Custom headers
    pub headers: Vec<(String, String)>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Email address with optional name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Format as RFC 5322
    pub fn to_rfc5322(&self) -> String {
        match &self.name {
            Some(name) => format!("{} <{}>", name, self.email),
            None => self.email.clone(),
        }
    }
}

impl EmailMessage {
    /// Create a new email message
    pub fn new(
        from: EmailAddress,
        to: Vec<EmailAddress>,
        subject: impl Into<String>,
        text_body: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: subject.into(),
            text_body: text_body.into(),
            html_body: None,
            headers: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Add HTML body
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html_body = Some(html.into());
        self
    }

    /// Add CC recipients
    pub fn cc(mut self, addresses: Vec<EmailAddress>) -> Self {
        self.cc = addresses;
        self
    }

    /// Set reply-to
    pub fn reply_to(mut self, address: EmailAddress) -> Self {
        self.reply_to = Some(address);
        self
    }

    /// Add a custom header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Add OpenProject-specific headers
    pub fn with_openproject_headers(mut self, project_id: Option<i64>, resource_id: i64) -> Self {
        self.headers.push(("X-OpenProject-Type".to_string(), "WorkPackage".to_string()));
        if let Some(pid) = project_id {
            self.headers.push(("X-OpenProject-Project".to_string(), pid.to_string()));
        }
        self.headers.push(("X-OpenProject-Id".to_string(), resource_id.to_string()));
        self
    }
}

/// Email sender trait
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Send an email
    async fn send(&self, message: &EmailMessage) -> EmailResult<String>;

    /// Check if the sender is configured
    fn is_configured(&self) -> bool;
}

/// Console email sender (for development)
pub struct ConsoleEmailSender;

impl Default for ConsoleEmailSender {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleEmailSender {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmailSender for ConsoleEmailSender {
    async fn send(&self, message: &EmailMessage) -> EmailResult<String> {
        println!("=== EMAIL ===");
        println!("From: {}", message.from.to_rfc5322());
        println!("To: {}", message.to.iter().map(|a| a.to_rfc5322()).collect::<Vec<_>>().join(", "));
        if !message.cc.is_empty() {
            println!("CC: {}", message.cc.iter().map(|a| a.to_rfc5322()).collect::<Vec<_>>().join(", "));
        }
        println!("Subject: {}", message.subject);
        println!("---");
        println!("{}", message.text_body);
        if let Some(ref html) = message.html_body {
            println!("--- HTML ---");
            println!("{}", html);
        }
        println!("=============");

        Ok(message.id.clone())
    }

    fn is_configured(&self) -> bool {
        true
    }
}

/// Email renderer for notifications
pub struct EmailRenderer {
    base_url: String,
    from_address: EmailAddress,
}

impl EmailRenderer {
    pub fn new(base_url: impl Into<String>, from_address: EmailAddress) -> Self {
        Self {
            base_url: base_url.into(),
            from_address,
        }
    }

    /// Render a notification as an email
    pub fn render_notification(
        &self,
        notification: &Notification,
        recipient_email: &str,
        recipient_name: Option<&str>,
    ) -> EmailMessage {
        let subject = self.render_subject(notification);
        let text_body = self.render_text_body(notification);
        let html_body = self.render_html_body(notification);

        let to = EmailAddress::new(recipient_email);
        let to = match recipient_name {
            Some(name) => to.with_name(name),
            None => to,
        };

        EmailMessage::new(
            self.from_address.clone(),
            vec![to],
            subject,
            text_body,
        )
        .with_html(html_body)
        .with_openproject_headers(notification.project_id, notification.resource_id)
    }

    fn render_subject(&self, notification: &Notification) -> String {
        match notification.notification_type {
            NotificationType::WorkPackageCreated => {
                format!("[OpenProject] Work Package #{} created", notification.resource_id)
            }
            NotificationType::WorkPackageUpdated => {
                format!("[OpenProject] Work Package #{} updated", notification.resource_id)
            }
            NotificationType::WorkPackageCommented => {
                format!("[OpenProject] New comment on Work Package #{}", notification.resource_id)
            }
            NotificationType::WorkPackageAssigned => {
                format!("[OpenProject] Work Package #{} assigned to you", notification.resource_id)
            }
            NotificationType::WorkPackageMentioned => {
                format!("[OpenProject] You were mentioned in Work Package #{}", notification.resource_id)
            }
            NotificationType::WorkPackageDueDateAlert => {
                format!("[OpenProject] Work Package #{} is due soon", notification.resource_id)
            }
            NotificationType::WorkPackageOverdue => {
                format!("[OpenProject] Work Package #{} is overdue", notification.resource_id)
            }
            NotificationType::MembershipAdded => {
                "[OpenProject] You have been added to a project".to_string()
            }
            _ => {
                format!("[OpenProject] {} notification", notification.resource_type)
            }
        }
    }

    fn render_text_body(&self, notification: &Notification) -> String {
        let mut body = String::new();

        body.push_str(&format!(
            "You have a new notification in OpenProject.\n\n"
        ));

        body.push_str(&format!(
            "Type: {:?}\n",
            notification.notification_type
        ));

        body.push_str(&format!(
            "Resource: {} #{}\n",
            notification.resource_type,
            notification.resource_id
        ));

        if let Some(actor_id) = notification.actor_id {
            body.push_str(&format!("By: User #{}\n", actor_id));
        }

        body.push_str(&format!(
            "\nView details: {}/work_packages/{}\n",
            self.base_url,
            notification.resource_id
        ));

        body.push_str("\n---\nYou received this email because you are subscribed to notifications.\n");

        body
    }

    fn render_html_body(&self, notification: &Notification) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #1a67a3; color: white; padding: 20px; }}
        .content {{ padding: 20px; background: #f5f5f5; }}
        .footer {{ padding: 20px; font-size: 12px; color: #666; }}
        .button {{ display: inline-block; padding: 10px 20px; background: #1a67a3; color: white; text-decoration: none; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>OpenProject Notification</h1>
        </div>
        <div class="content">
            <p>{} #{}</p>
            <p><a class="button" href="{}/work_packages/{}">View in OpenProject</a></p>
        </div>
        <div class="footer">
            <p>You received this email because you are subscribed to notifications.</p>
        </div>
    </div>
</body>
</html>"#,
            notification.resource_type,
            notification.resource_id,
            self.base_url,
            notification.resource_id
        )
    }
}

/// Email digest builder
pub struct DigestBuilder {
    notifications: Vec<Notification>,
    renderer: EmailRenderer,
}

impl DigestBuilder {
    pub fn new(renderer: EmailRenderer) -> Self {
        Self {
            notifications: Vec::new(),
            renderer,
        }
    }

    pub fn add(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    pub fn build(
        &self,
        recipient_email: &str,
        recipient_name: Option<&str>,
        period: &str,
    ) -> Option<EmailMessage> {
        if self.notifications.is_empty() {
            return None;
        }

        let subject = format!("[OpenProject] Your {} digest ({} notifications)", period, self.notifications.len());

        let mut text_body = format!(
            "Here's your {} OpenProject digest with {} notifications:\n\n",
            period,
            self.notifications.len()
        );

        for notification in &self.notifications {
            text_body.push_str(&format!(
                "- {:?}: {} #{}\n",
                notification.notification_type,
                notification.resource_type,
                notification.resource_id
            ));
        }

        let to = EmailAddress::new(recipient_email);
        let to = match recipient_name {
            Some(name) => to.with_name(name),
            None => to,
        };

        Some(EmailMessage::new(
            self.renderer.from_address.clone(),
            vec![to],
            subject,
            text_body,
        ))
    }
}

/// Microsoft Graph email sender configuration
#[derive(Debug, Clone)]
pub struct MsGraphConfig {
    /// Azure AD tenant ID
    pub tenant_id: String,
    /// Azure AD application (client) ID
    pub client_id: String,
    /// Azure AD client secret
    pub client_secret: String,
    /// User principal name or object ID of the sender
    /// (e.g., noreply@yourdomain.com or user object ID)
    pub sender: String,
}

impl MsGraphConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Option<Self> {
        Some(Self {
            tenant_id: std::env::var("MS_GRAPH_TENANT_ID").ok()?,
            client_id: std::env::var("MS_GRAPH_CLIENT_ID").ok()?,
            client_secret: std::env::var("MS_GRAPH_CLIENT_SECRET").ok()?,
            sender: std::env::var("MS_GRAPH_SENDER").ok()?,
        })
    }
}

/// Microsoft Graph email sender
///
/// Uses Azure AD app registration with Mail.Send permission to send emails
/// via the Microsoft Graph API.
///
/// Required Azure AD app permissions (Application type):
/// - Microsoft Graph > Mail.Send
///
/// Environment variables:
/// - MS_GRAPH_TENANT_ID: Azure AD tenant ID
/// - MS_GRAPH_CLIENT_ID: Application (client) ID
/// - MS_GRAPH_CLIENT_SECRET: Client secret value
/// - MS_GRAPH_SENDER: Sender email (e.g., noreply@yourdomain.com)
pub struct MsGraphEmailSender {
    config: MsGraphConfig,
    // In production, this would use reqwest or similar HTTP client
    // and cache the OAuth token
}

impl MsGraphEmailSender {
    pub fn new(config: MsGraphConfig) -> Self {
        tracing::info!(
            sender = %config.sender,
            tenant = %config.tenant_id,
            "Microsoft Graph email sender initialized"
        );
        Self { config }
    }

    /// Create from environment variables
    pub fn from_env() -> Option<Self> {
        MsGraphConfig::from_env().map(Self::new)
    }

    /// Get OAuth2 access token from Azure AD
    async fn get_access_token(&self) -> EmailResult<String> {
        // Token endpoint
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.config.tenant_id
        );

        // In a real implementation, this would:
        // 1. POST to token_url with:
        //    - grant_type=client_credentials
        //    - client_id={client_id}
        //    - client_secret={client_secret}
        //    - scope=https://graph.microsoft.com/.default
        // 2. Parse the JSON response for access_token
        // 3. Cache the token until it expires

        tracing::debug!(url = %token_url, "Would fetch OAuth token");

        // Placeholder - in production use reqwest
        Err(EmailError::SendFailed(
            "MS Graph requires HTTP client implementation (reqwest)".to_string()
        ))
    }

    /// Build the Graph API message payload
    fn build_message_payload(&self, message: &EmailMessage) -> serde_json::Value {
        let to_recipients: Vec<serde_json::Value> = message.to.iter().map(|addr| {
            serde_json::json!({
                "emailAddress": {
                    "address": addr.email,
                    "name": addr.name
                }
            })
        }).collect();

        let cc_recipients: Vec<serde_json::Value> = message.cc.iter().map(|addr| {
            serde_json::json!({
                "emailAddress": {
                    "address": addr.email,
                    "name": addr.name
                }
            })
        }).collect();

        let bcc_recipients: Vec<serde_json::Value> = message.bcc.iter().map(|addr| {
            serde_json::json!({
                "emailAddress": {
                    "address": addr.email,
                    "name": addr.name
                }
            })
        }).collect();

        let mut body = serde_json::json!({
            "message": {
                "subject": message.subject,
                "body": {
                    "contentType": if message.html_body.is_some() { "HTML" } else { "Text" },
                    "content": message.html_body.as_ref().unwrap_or(&message.text_body)
                },
                "toRecipients": to_recipients,
                "ccRecipients": cc_recipients,
                "bccRecipients": bcc_recipients
            },
            "saveToSentItems": "true"
        });

        // Add reply-to if specified
        if let Some(ref reply_to) = message.reply_to {
            body["message"]["replyTo"] = serde_json::json!([{
                "emailAddress": {
                    "address": reply_to.email,
                    "name": reply_to.name
                }
            }]);
        }

        // Add custom headers
        if !message.headers.is_empty() {
            let internet_headers: Vec<serde_json::Value> = message.headers.iter().map(|(name, value)| {
                serde_json::json!({
                    "name": name,
                    "value": value
                })
            }).collect();
            body["message"]["internetMessageHeaders"] = serde_json::json!(internet_headers);
        }

        body
    }
}

#[async_trait]
impl EmailSender for MsGraphEmailSender {
    async fn send(&self, message: &EmailMessage) -> EmailResult<String> {
        tracing::info!(
            to = ?message.to.first().map(|a| &a.email),
            subject = %message.subject,
            "Sending email via Microsoft Graph"
        );

        // Get access token
        let _token = self.get_access_token().await?;

        // Build the sendMail endpoint URL
        let send_url = format!(
            "https://graph.microsoft.com/v1.0/users/{}/sendMail",
            self.config.sender
        );

        // Build message payload
        let _payload = self.build_message_payload(message);

        tracing::debug!(url = %send_url, "Would POST to Graph API");

        // In a real implementation, this would:
        // 1. POST to send_url with:
        //    - Authorization: Bearer {token}
        //    - Content-Type: application/json
        //    - Body: payload
        // 2. Check for 202 Accepted response
        // 3. Return message ID

        // Placeholder response
        Err(EmailError::SendFailed(
            "MS Graph requires HTTP client implementation (reqwest)".to_string()
        ))
    }

    fn is_configured(&self) -> bool {
        !self.config.tenant_id.is_empty()
            && !self.config.client_id.is_empty()
            && !self.config.client_secret.is_empty()
            && !self.config.sender.is_empty()
    }
}

/// Memory-based email sender for testing
pub struct MemoryEmailSender {
    sent: std::sync::Arc<tokio::sync::RwLock<Vec<EmailMessage>>>,
}

impl Default for MemoryEmailSender {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryEmailSender {
    pub fn new() -> Self {
        Self {
            sent: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Get all sent messages
    pub async fn sent_messages(&self) -> Vec<EmailMessage> {
        self.sent.read().await.clone()
    }

    /// Clear sent messages
    pub async fn clear(&self) {
        self.sent.write().await.clear();
    }
}

#[async_trait]
impl EmailSender for MemoryEmailSender {
    async fn send(&self, message: &EmailMessage) -> EmailResult<String> {
        self.sent.write().await.push(message.clone());
        Ok(message.id.clone())
    }

    fn is_configured(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::NotificationReason;

    #[test]
    fn test_email_address_format() {
        let addr = EmailAddress::new("test@example.com").with_name("Test User");
        assert_eq!(addr.to_rfc5322(), "Test User <test@example.com>");

        let addr2 = EmailAddress::new("no-name@example.com");
        assert_eq!(addr2.to_rfc5322(), "no-name@example.com");
    }

    #[test]
    fn test_email_message_creation() {
        let from = EmailAddress::new("noreply@openproject.com").with_name("OpenProject");
        let to = vec![EmailAddress::new("user@example.com")];

        let message = EmailMessage::new(from, to, "Test Subject", "Test body")
            .with_html("<p>Test body</p>")
            .header("X-Custom", "value");

        assert_eq!(message.subject, "Test Subject");
        assert!(message.html_body.is_some());
        assert_eq!(message.headers.len(), 1);
    }

    #[test]
    fn test_email_renderer() {
        let from = EmailAddress::new("noreply@openproject.com").with_name("OpenProject");
        let renderer = EmailRenderer::new("https://openproject.example.com", from);

        let notification = Notification::work_package(
            1,
            NotificationType::WorkPackageUpdated,
            NotificationReason::Assigned,
            100,
        );

        let email = renderer.render_notification(&notification, "user@example.com", Some("Test User"));

        assert!(email.subject.contains("100"));
        assert!(email.subject.contains("updated"));
        assert!(email.text_body.contains("https://openproject.example.com"));
    }

    #[tokio::test]
    async fn test_console_sender() {
        let sender = ConsoleEmailSender::new();
        let from = EmailAddress::new("test@example.com");
        let to = vec![EmailAddress::new("user@example.com")];

        let message = EmailMessage::new(from, to, "Test", "Test body");
        let result = sender.send(&message).await;

        assert!(result.is_ok());
    }
}
