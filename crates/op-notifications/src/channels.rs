//! Notification Delivery Channels
//!
//! Mirrors: app/services/notifications/create_service.rb

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::notification::Notification;

/// Channel errors
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
    #[error("Channel disabled")]
    Disabled,
    #[error("Rate limited")]
    RateLimited,
    #[error("Invalid recipient")]
    InvalidRecipient,
}

pub type ChannelResult<T> = Result<T, ChannelError>;

/// Notification delivery channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    /// In-app notification (bell icon)
    InApp,
    /// Email notification
    Email,
    /// Push notification (mobile)
    Push,
    /// Webhook
    Webhook,
    /// Slack integration
    Slack,
    /// Microsoft Teams integration
    Teams,
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel: Channel,
    pub enabled: bool,
    pub settings: serde_json::Value,
}

impl ChannelConfig {
    pub fn in_app() -> Self {
        Self {
            channel: Channel::InApp,
            enabled: true,
            settings: serde_json::json!({}),
        }
    }

    pub fn email() -> Self {
        Self {
            channel: Channel::Email,
            enabled: true,
            settings: serde_json::json!({}),
        }
    }

    pub fn webhook(url: impl Into<String>) -> Self {
        Self {
            channel: Channel::Webhook,
            enabled: true,
            settings: serde_json::json!({
                "url": url.into(),
            }),
        }
    }
}

/// Delivery result for a single channel
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub channel: Channel,
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

impl DeliveryResult {
    pub fn success(channel: Channel, message_id: impl Into<String>) -> Self {
        Self {
            channel,
            success: true,
            message_id: Some(message_id.into()),
            error: None,
        }
    }

    pub fn failure(channel: Channel, error: impl Into<String>) -> Self {
        Self {
            channel,
            success: false,
            message_id: None,
            error: Some(error.into()),
        }
    }
}

/// Channel handler trait
#[async_trait]
pub trait ChannelHandler: Send + Sync {
    /// Get the channel type
    fn channel(&self) -> Channel;

    /// Check if the channel is available
    fn is_available(&self) -> bool;

    /// Deliver a notification
    async fn deliver(&self, notification: &Notification) -> ChannelResult<DeliveryResult>;
}

/// In-app channel handler (stores in database)
pub struct InAppChannel {
    enabled: bool,
}

impl Default for InAppChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl InAppChannel {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

#[async_trait]
impl ChannelHandler for InAppChannel {
    fn channel(&self) -> Channel {
        Channel::InApp
    }

    fn is_available(&self) -> bool {
        self.enabled
    }

    async fn deliver(&self, notification: &Notification) -> ChannelResult<DeliveryResult> {
        if !self.enabled {
            return Err(ChannelError::Disabled);
        }

        // In a real implementation, this would store to database
        // For now, we just return success
        Ok(DeliveryResult::success(
            Channel::InApp,
            notification.id.map(|id| id.to_string()).unwrap_or_else(|| "pending".to_string()),
        ))
    }
}

/// Webhook channel handler
pub struct WebhookChannel {
    url: String,
    secret: Option<String>,
    enabled: bool,
}

impl WebhookChannel {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            secret: None,
            enabled: true,
        }
    }

    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }
}

#[async_trait]
impl ChannelHandler for WebhookChannel {
    fn channel(&self) -> Channel {
        Channel::Webhook
    }

    fn is_available(&self) -> bool {
        self.enabled && !self.url.is_empty()
    }

    async fn deliver(&self, notification: &Notification) -> ChannelResult<DeliveryResult> {
        if !self.is_available() {
            return Err(ChannelError::Disabled);
        }

        // In a real implementation, this would make an HTTP POST request
        // For now, simulate delivery
        let payload = serde_json::json!({
            "event": "notification",
            "notification_type": notification.notification_type,
            "resource_type": notification.resource_type,
            "resource_id": notification.resource_id,
            "recipient_id": notification.recipient_id,
            "actor_id": notification.actor_id,
            "created_at": notification.created_at.to_rfc3339(),
        });

        tracing::info!(
            "Webhook delivery to {}: {}",
            self.url,
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );

        Ok(DeliveryResult::success(
            Channel::Webhook,
            format!("webhook-{}", uuid::Uuid::new_v4()),
        ))
    }
}

/// Multi-channel dispatcher
pub struct ChannelDispatcher {
    handlers: Vec<Box<dyn ChannelHandler>>,
}

impl Default for ChannelDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a channel handler
    pub fn add_handler<H: ChannelHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    /// Register the default channels
    pub fn with_defaults(mut self) -> Self {
        self.add_handler(InAppChannel::new());
        self
    }

    /// Deliver a notification to all channels
    pub async fn deliver_all(&self, notification: &Notification) -> Vec<DeliveryResult> {
        let mut results = Vec::new();

        for handler in &self.handlers {
            if handler.is_available() {
                match handler.deliver(notification).await {
                    Ok(result) => results.push(result),
                    Err(e) => results.push(DeliveryResult::failure(handler.channel(), e.to_string())),
                }
            }
        }

        results
    }

    /// Deliver to a specific channel
    pub async fn deliver_to(
        &self,
        channel: Channel,
        notification: &Notification,
    ) -> ChannelResult<DeliveryResult> {
        for handler in &self.handlers {
            if handler.channel() == channel {
                return handler.deliver(notification).await;
            }
        }

        Err(ChannelError::Disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::{NotificationReason, NotificationType};

    #[tokio::test]
    async fn test_in_app_channel() {
        let channel = InAppChannel::new();
        let notification = Notification::work_package(
            1,
            NotificationType::WorkPackageCreated,
            NotificationReason::Watched,
            100,
        );

        let result = channel.deliver(&notification).await.unwrap();
        assert!(result.success);
        assert_eq!(result.channel, Channel::InApp);
    }

    #[tokio::test]
    async fn test_channel_dispatcher() {
        let mut dispatcher = ChannelDispatcher::new();
        dispatcher.add_handler(InAppChannel::new());

        let notification = Notification::work_package(
            1,
            NotificationType::WorkPackageUpdated,
            NotificationReason::Assigned,
            100,
        );

        let results = dispatcher.deliver_all(&notification).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[test]
    fn test_channel_config() {
        let config = ChannelConfig::webhook("https://example.com/webhook");
        assert_eq!(config.channel, Channel::Webhook);
        assert!(config.enabled);
    }
}
