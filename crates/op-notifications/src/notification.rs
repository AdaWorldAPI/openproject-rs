//! Notification Model
//!
//! Mirrors: app/models/notification.rb

use chrono::{DateTime, Utc};
use op_core::traits::Id;
use serde::{Deserialize, Serialize};

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Work package created
    WorkPackageCreated,
    /// Work package updated
    WorkPackageUpdated,
    /// Work package commented
    WorkPackageCommented,
    /// Work package assigned
    WorkPackageAssigned,
    /// Work package mentioned
    WorkPackageMentioned,
    /// Work package due date approaching
    WorkPackageDueDateAlert,
    /// Work package overdue
    WorkPackageOverdue,
    /// Project created
    ProjectCreated,
    /// Membership added
    MembershipAdded,
    /// Membership updated
    MembershipUpdated,
    /// Document added
    DocumentAdded,
    /// News added
    NewsAdded,
    /// Wiki page updated
    WikiPageUpdated,
    /// Message posted
    MessagePosted,
    /// File uploaded
    FileUploaded,
    /// Meeting invitation
    MeetingInvitation,
    /// Reminder
    Reminder,
}

impl NotificationType {
    /// Get the i18n key for this notification type
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::WorkPackageCreated => "notification.work_package.created",
            Self::WorkPackageUpdated => "notification.work_package.updated",
            Self::WorkPackageCommented => "notification.work_package.commented",
            Self::WorkPackageAssigned => "notification.work_package.assigned",
            Self::WorkPackageMentioned => "notification.work_package.mentioned",
            Self::WorkPackageDueDateAlert => "notification.work_package.due_date_alert",
            Self::WorkPackageOverdue => "notification.work_package.overdue",
            Self::ProjectCreated => "notification.project.created",
            Self::MembershipAdded => "notification.membership.added",
            Self::MembershipUpdated => "notification.membership.updated",
            Self::DocumentAdded => "notification.document.added",
            Self::NewsAdded => "notification.news.added",
            Self::WikiPageUpdated => "notification.wiki.updated",
            Self::MessagePosted => "notification.message.posted",
            Self::FileUploaded => "notification.file.uploaded",
            Self::MeetingInvitation => "notification.meeting.invitation",
            Self::Reminder => "notification.reminder",
        }
    }

    /// Check if this is a work package notification
    pub fn is_work_package(&self) -> bool {
        matches!(
            self,
            Self::WorkPackageCreated
                | Self::WorkPackageUpdated
                | Self::WorkPackageCommented
                | Self::WorkPackageAssigned
                | Self::WorkPackageMentioned
                | Self::WorkPackageDueDateAlert
                | Self::WorkPackageOverdue
        )
    }
}

/// Reason for the notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationReason {
    /// User is assigned to the resource
    Assigned,
    /// User is responsible for the resource
    Responsible,
    /// User is watching the resource
    Watched,
    /// User was mentioned
    Mentioned,
    /// User subscribed to notifications
    Subscribed,
    /// User is involved (commenter, etc.)
    Involved,
    /// User is project member with notifications enabled
    ProjectMember,
    /// Date-based alert
    DateAlert,
    /// System notification
    System,
}

/// A notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification ID
    pub id: Option<Id>,
    /// Recipient user ID
    pub recipient_id: Id,
    /// Actor (who triggered this) user ID
    pub actor_id: Option<Id>,
    /// Notification type
    pub notification_type: NotificationType,
    /// Reason for this notification
    pub reason: NotificationReason,
    /// Related resource type
    pub resource_type: String,
    /// Related resource ID
    pub resource_id: Id,
    /// Project ID (if applicable)
    pub project_id: Option<Id>,
    /// Journal ID (for change notifications)
    pub journal_id: Option<Id>,
    /// Has been read
    pub read_at: Option<DateTime<Utc>>,
    /// Has been emailed
    pub mail_sent_at: Option<DateTime<Utc>>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        recipient_id: Id,
        notification_type: NotificationType,
        reason: NotificationReason,
        resource_type: impl Into<String>,
        resource_id: Id,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            recipient_id,
            actor_id: None,
            notification_type,
            reason,
            resource_type: resource_type.into(),
            resource_id,
            project_id: None,
            journal_id: None,
            read_at: None,
            mail_sent_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a work package notification
    pub fn work_package(
        recipient_id: Id,
        notification_type: NotificationType,
        reason: NotificationReason,
        work_package_id: Id,
    ) -> Self {
        Self::new(
            recipient_id,
            notification_type,
            reason,
            "WorkPackage",
            work_package_id,
        )
    }

    /// Set the actor
    pub fn with_actor(mut self, actor_id: Id) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    /// Set the project
    pub fn with_project(mut self, project_id: Id) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Set the journal
    pub fn with_journal(mut self, journal_id: Id) -> Self {
        self.journal_id = Some(journal_id);
        self
    }

    /// Check if the notification is unread
    pub fn is_unread(&self) -> bool {
        self.read_at.is_none()
    }

    /// Check if email has been sent
    pub fn is_mail_sent(&self) -> bool {
        self.mail_sent_at.is_some()
    }

    /// Mark as read
    pub fn mark_read(&mut self) {
        self.read_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark as mail sent
    pub fn mark_mail_sent(&mut self) {
        self.mail_sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

/// Notification settings for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub user_id: Id,
    /// Receive in-app notifications
    pub in_app_enabled: bool,
    /// Receive email notifications
    pub email_enabled: bool,
    /// Email frequency
    pub email_frequency: EmailFrequency,
    /// Notification types to receive
    pub enabled_types: Vec<NotificationType>,
    /// Reasons that trigger notifications
    pub enabled_reasons: Vec<NotificationReason>,
    /// Specific projects to watch (None = all projects)
    pub watched_projects: Option<Vec<Id>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailFrequency {
    Immediate,
    Daily,
    Weekly,
    Never,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            user_id: 0,
            in_app_enabled: true,
            email_enabled: true,
            email_frequency: EmailFrequency::Immediate,
            enabled_types: vec![
                NotificationType::WorkPackageAssigned,
                NotificationType::WorkPackageMentioned,
                NotificationType::WorkPackageCommented,
                NotificationType::MembershipAdded,
            ],
            enabled_reasons: vec![
                NotificationReason::Assigned,
                NotificationReason::Mentioned,
                NotificationReason::Watched,
            ],
            watched_projects: None,
        }
    }
}

impl NotificationSettings {
    pub fn for_user(user_id: Id) -> Self {
        Self {
            user_id,
            ..Default::default()
        }
    }

    /// Check if a notification should be sent
    pub fn should_notify(
        &self,
        notification_type: NotificationType,
        reason: NotificationReason,
        project_id: Option<Id>,
    ) -> bool {
        // Check if in-app is enabled
        if !self.in_app_enabled {
            return false;
        }

        // Check notification type
        if !self.enabled_types.contains(&notification_type) {
            return false;
        }

        // Check reason
        if !self.enabled_reasons.contains(&reason) {
            return false;
        }

        // Check project filter
        if let Some(ref watched) = self.watched_projects {
            if let Some(pid) = project_id {
                if !watched.contains(&pid) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if email should be sent
    pub fn should_email(&self) -> bool {
        self.email_enabled && self.email_frequency != EmailFrequency::Never
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::work_package(
            1,
            NotificationType::WorkPackageAssigned,
            NotificationReason::Assigned,
            100,
        )
        .with_actor(2)
        .with_project(10);

        assert_eq!(notification.recipient_id, 1);
        assert_eq!(notification.actor_id, Some(2));
        assert_eq!(notification.project_id, Some(10));
        assert!(notification.is_unread());
    }

    #[test]
    fn test_mark_read() {
        let mut notification = Notification::work_package(
            1,
            NotificationType::WorkPackageUpdated,
            NotificationReason::Watched,
            100,
        );

        assert!(notification.is_unread());
        notification.mark_read();
        assert!(!notification.is_unread());
    }

    #[test]
    fn test_notification_settings() {
        let settings = NotificationSettings::for_user(1);

        assert!(settings.should_notify(
            NotificationType::WorkPackageAssigned,
            NotificationReason::Assigned,
            Some(1),
        ));

        assert!(!settings.should_notify(
            NotificationType::DocumentAdded,
            NotificationReason::Assigned,
            Some(1),
        ));
    }

    #[test]
    fn test_project_filter() {
        let mut settings = NotificationSettings::for_user(1);
        settings.watched_projects = Some(vec![1, 2, 3]);

        assert!(settings.should_notify(
            NotificationType::WorkPackageAssigned,
            NotificationReason::Assigned,
            Some(1),
        ));

        assert!(!settings.should_notify(
            NotificationType::WorkPackageAssigned,
            NotificationReason::Assigned,
            Some(99),
        ));
    }
}
