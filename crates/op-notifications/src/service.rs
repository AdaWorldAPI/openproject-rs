//! Notification Service
//!
//! Orchestrates notification creation, storage, and delivery.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::channels::{Channel, ChannelDispatcher, DeliveryResult};
use crate::email::{EmailRenderer, EmailSender};
use crate::jobs::{Job, JobQueue};
use crate::notification::{
    EmailFrequency, Notification, NotificationReason, NotificationSettings, NotificationType,
};

/// Service errors
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Notification not found: {0}")]
    NotFound(Id),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Delivery error: {0}")]
    DeliveryError(String),
    #[error("Job queue error: {0}")]
    JobError(String),
}

pub type ServiceResult<T> = Result<T, ServiceError>;

/// Event emitted when a notification is created/delivered
#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub notification: Notification,
    pub delivery_results: Vec<DeliveryResult>,
    pub timestamp: DateTime<Utc>,
}

/// Notification storage trait
#[async_trait]
pub trait NotificationStore: Send + Sync {
    /// Create a notification
    async fn create(&self, notification: &mut Notification) -> ServiceResult<Id>;

    /// Get a notification by ID
    async fn get(&self, id: Id) -> ServiceResult<Option<Notification>>;

    /// Get notifications for a user
    async fn get_for_user(
        &self,
        user_id: Id,
        unread_only: bool,
        limit: usize,
    ) -> ServiceResult<Vec<Notification>>;

    /// Get user's notification settings
    async fn get_settings(&self, user_id: Id) -> ServiceResult<NotificationSettings>;

    /// Update a notification
    async fn update(&self, notification: &Notification) -> ServiceResult<()>;

    /// Delete a notification
    async fn delete(&self, id: Id) -> ServiceResult<()>;

    /// Mark all as read for a user
    async fn mark_all_read(&self, user_id: Id) -> ServiceResult<usize>;

    /// Get unread count for a user
    async fn unread_count(&self, user_id: Id) -> ServiceResult<usize>;

    /// Get pending email notifications for digest
    async fn get_pending_digest(
        &self,
        user_id: Id,
        frequency: EmailFrequency,
    ) -> ServiceResult<Vec<Notification>>;
}

/// In-memory notification store for development/testing
pub struct MemoryNotificationStore {
    notifications: RwLock<Vec<Notification>>,
    settings: RwLock<std::collections::HashMap<Id, NotificationSettings>>,
    next_id: std::sync::atomic::AtomicI64,
}

impl Default for MemoryNotificationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryNotificationStore {
    pub fn new() -> Self {
        Self {
            notifications: RwLock::new(Vec::new()),
            settings: RwLock::new(std::collections::HashMap::new()),
            next_id: std::sync::atomic::AtomicI64::new(1),
        }
    }
}

#[async_trait]
impl NotificationStore for MemoryNotificationStore {
    async fn create(&self, notification: &mut Notification) -> ServiceResult<Id> {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        notification.id = Some(id);

        let mut notifications = self.notifications.write().await;
        notifications.push(notification.clone());

        Ok(id)
    }

    async fn get(&self, id: Id) -> ServiceResult<Option<Notification>> {
        let notifications = self.notifications.read().await;
        Ok(notifications.iter().find(|n| n.id == Some(id)).cloned())
    }

    async fn get_for_user(
        &self,
        user_id: Id,
        unread_only: bool,
        limit: usize,
    ) -> ServiceResult<Vec<Notification>> {
        let notifications = self.notifications.read().await;
        Ok(notifications
            .iter()
            .filter(|n| n.recipient_id == user_id)
            .filter(|n| !unread_only || n.is_unread())
            .take(limit)
            .cloned()
            .collect())
    }

    async fn get_settings(&self, user_id: Id) -> ServiceResult<NotificationSettings> {
        let settings = self.settings.read().await;
        Ok(settings
            .get(&user_id)
            .cloned()
            .unwrap_or_else(|| NotificationSettings::for_user(user_id)))
    }

    async fn update(&self, notification: &Notification) -> ServiceResult<()> {
        let mut notifications = self.notifications.write().await;
        if let Some(pos) = notifications.iter().position(|n| n.id == notification.id) {
            notifications[pos] = notification.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: Id) -> ServiceResult<()> {
        let mut notifications = self.notifications.write().await;
        notifications.retain(|n| n.id != Some(id));
        Ok(())
    }

    async fn mark_all_read(&self, user_id: Id) -> ServiceResult<usize> {
        let mut notifications = self.notifications.write().await;
        let mut count = 0;

        for notification in notifications.iter_mut() {
            if notification.recipient_id == user_id && notification.is_unread() {
                notification.mark_read();
                count += 1;
            }
        }

        Ok(count)
    }

    async fn unread_count(&self, user_id: Id) -> ServiceResult<usize> {
        let notifications = self.notifications.read().await;
        Ok(notifications
            .iter()
            .filter(|n| n.recipient_id == user_id && n.is_unread())
            .count())
    }

    async fn get_pending_digest(
        &self,
        user_id: Id,
        _frequency: EmailFrequency,
    ) -> ServiceResult<Vec<Notification>> {
        let notifications = self.notifications.read().await;
        Ok(notifications
            .iter()
            .filter(|n| n.recipient_id == user_id && !n.is_mail_sent())
            .cloned()
            .collect())
    }
}

/// Notification service
pub struct NotificationService<S: NotificationStore, Q: JobQueue, E: EmailSender> {
    store: Arc<S>,
    job_queue: Arc<Q>,
    email_sender: Arc<E>,
    dispatcher: ChannelDispatcher,
    email_renderer: EmailRenderer,
}

impl<S: NotificationStore, Q: JobQueue, E: EmailSender> NotificationService<S, Q, E> {
    pub fn new(
        store: Arc<S>,
        job_queue: Arc<Q>,
        email_sender: Arc<E>,
        email_renderer: EmailRenderer,
    ) -> Self {
        Self {
            store,
            job_queue,
            email_sender,
            dispatcher: ChannelDispatcher::new().with_defaults(),
            email_renderer,
        }
    }

    /// Create and send a notification
    pub async fn notify(
        &self,
        recipient_id: Id,
        notification_type: NotificationType,
        reason: NotificationReason,
        resource_type: impl Into<String>,
        resource_id: Id,
        actor_id: Option<Id>,
        project_id: Option<Id>,
    ) -> ServiceResult<NotificationEvent> {
        // Get user settings
        let settings = self.store.get_settings(recipient_id).await?;

        // Check if notification should be sent
        if !settings.should_notify(notification_type, reason, project_id) {
            return Err(ServiceError::DeliveryError("User has disabled this notification type".into()));
        }

        // Create notification
        let mut notification = Notification::new(
            recipient_id,
            notification_type,
            reason,
            resource_type,
            resource_id,
        );

        if let Some(aid) = actor_id {
            notification = notification.with_actor(aid);
        }

        if let Some(pid) = project_id {
            notification = notification.with_project(pid);
        }

        // Store notification
        let id = self.store.create(&mut notification).await?;
        notification.id = Some(id);

        // Deliver to channels
        let delivery_results = self.dispatcher.deliver_all(&notification).await;

        // Queue email if needed
        if settings.should_email() && settings.email_frequency == EmailFrequency::Immediate {
            self.queue_email(&notification).await?;
        }

        Ok(NotificationEvent {
            notification,
            delivery_results,
            timestamp: Utc::now(),
        })
    }

    /// Queue an email for delivery
    async fn queue_email(&self, notification: &Notification) -> ServiceResult<()> {
        let job = Job::new(
            "send_notification_email",
            serde_json::json!({
                "notification_id": notification.id,
            }),
        )
        .queue("mailers");

        self.job_queue
            .enqueue(job)
            .await
            .map_err(|e| ServiceError::JobError(e.to_string()))?;

        Ok(())
    }

    /// Get notifications for a user
    pub async fn get_notifications(
        &self,
        user_id: Id,
        unread_only: bool,
        limit: usize,
    ) -> ServiceResult<Vec<Notification>> {
        self.store.get_for_user(user_id, unread_only, limit).await
    }

    /// Get unread count
    pub async fn unread_count(&self, user_id: Id) -> ServiceResult<usize> {
        self.store.unread_count(user_id).await
    }

    /// Mark notification as read
    pub async fn mark_read(&self, notification_id: Id) -> ServiceResult<()> {
        let mut notification = self
            .store
            .get(notification_id)
            .await?
            .ok_or(ServiceError::NotFound(notification_id))?;

        notification.mark_read();
        self.store.update(&notification).await
    }

    /// Mark all notifications as read
    pub async fn mark_all_read(&self, user_id: Id) -> ServiceResult<usize> {
        self.store.mark_all_read(user_id).await
    }

    /// Delete a notification
    pub async fn delete(&self, notification_id: Id) -> ServiceResult<()> {
        self.store.delete(notification_id).await
    }

    /// Send daily digest emails
    pub async fn send_daily_digest(&self) -> ServiceResult<usize> {
        // In a real implementation, this would iterate over all users
        // with daily digest enabled and send their digest
        Ok(0)
    }

    /// Send weekly digest emails
    pub async fn send_weekly_digest(&self) -> ServiceResult<usize> {
        Ok(0)
    }
}

/// Work package notification helper
pub struct WorkPackageNotifier<S: NotificationStore, Q: JobQueue, E: EmailSender> {
    service: Arc<NotificationService<S, Q, E>>,
}

impl<S: NotificationStore, Q: JobQueue, E: EmailSender> WorkPackageNotifier<S, Q, E> {
    pub fn new(service: Arc<NotificationService<S, Q, E>>) -> Self {
        Self { service }
    }

    /// Notify about work package creation
    pub async fn on_created(
        &self,
        work_package_id: Id,
        project_id: Id,
        author_id: Id,
        watchers: Vec<Id>,
    ) -> Vec<ServiceResult<NotificationEvent>> {
        let mut results = Vec::new();

        for watcher_id in watchers {
            if watcher_id != author_id {
                let result = self
                    .service
                    .notify(
                        watcher_id,
                        NotificationType::WorkPackageCreated,
                        NotificationReason::Watched,
                        "WorkPackage",
                        work_package_id,
                        Some(author_id),
                        Some(project_id),
                    )
                    .await;
                results.push(result);
            }
        }

        results
    }

    /// Notify about work package assignment
    pub async fn on_assigned(
        &self,
        work_package_id: Id,
        project_id: Id,
        actor_id: Id,
        assignee_id: Id,
    ) -> ServiceResult<NotificationEvent> {
        self.service
            .notify(
                assignee_id,
                NotificationType::WorkPackageAssigned,
                NotificationReason::Assigned,
                "WorkPackage",
                work_package_id,
                Some(actor_id),
                Some(project_id),
            )
            .await
    }

    /// Notify about mentions in a work package
    pub async fn on_mentioned(
        &self,
        work_package_id: Id,
        project_id: Id,
        actor_id: Id,
        mentioned_user_ids: Vec<Id>,
    ) -> Vec<ServiceResult<NotificationEvent>> {
        let mut results = Vec::new();

        for user_id in mentioned_user_ids {
            if user_id != actor_id {
                let result = self
                    .service
                    .notify(
                        user_id,
                        NotificationType::WorkPackageMentioned,
                        NotificationReason::Mentioned,
                        "WorkPackage",
                        work_package_id,
                        Some(actor_id),
                        Some(project_id),
                    )
                    .await;
                results.push(result);
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email::ConsoleEmailSender;
    use crate::jobs::MemoryJobQueue;

    fn create_test_store() -> Arc<MemoryNotificationStore> {
        Arc::new(MemoryNotificationStore::new())
    }

    #[tokio::test]
    async fn test_memory_store_create() {
        let store = create_test_store();

        let mut notification = Notification::work_package(
            1,
            NotificationType::WorkPackageCreated,
            NotificationReason::Watched,
            100,
        );

        let id = store.create(&mut notification).await.unwrap();
        assert!(id > 0);
        assert_eq!(notification.id, Some(id));
    }

    #[tokio::test]
    async fn test_memory_store_get_for_user() {
        let store = create_test_store();

        for i in 0..3 {
            let mut notification = Notification::work_package(
                1,
                NotificationType::WorkPackageUpdated,
                NotificationReason::Watched,
                100 + i,
            );
            store.create(&mut notification).await.unwrap();
        }

        let notifications = store.get_for_user(1, false, 10).await.unwrap();
        assert_eq!(notifications.len(), 3);
    }

    #[tokio::test]
    async fn test_memory_store_mark_all_read() {
        let store = create_test_store();

        for i in 0..3 {
            let mut notification = Notification::work_package(
                1,
                NotificationType::WorkPackageUpdated,
                NotificationReason::Watched,
                100 + i,
            );
            store.create(&mut notification).await.unwrap();
        }

        let unread = store.unread_count(1).await.unwrap();
        assert_eq!(unread, 3);

        let marked = store.mark_all_read(1).await.unwrap();
        assert_eq!(marked, 3);

        let unread = store.unread_count(1).await.unwrap();
        assert_eq!(unread, 0);
    }

    #[tokio::test]
    async fn test_memory_store_settings() {
        let store = create_test_store();

        let settings = store.get_settings(1).await.unwrap();
        assert_eq!(settings.user_id, 1);
        assert!(settings.in_app_enabled);
    }
}
