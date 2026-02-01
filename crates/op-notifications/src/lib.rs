//! # op-notifications
//!
//! Notifications and background job processing for OpenProject RS.
//!
//! ## Features
//!
//! - Background job queue with retry support
//! - In-app notifications (bell icon)
//! - Email notifications
//! - Digest emails (daily/weekly)
//! - Mention notifications

pub mod jobs;
pub mod notification;
pub mod channels;
pub mod email;
pub mod service;

pub use jobs::{Job, JobQueue, JobStatus, JobError, MemoryJobQueue};
pub use notification::{Notification, NotificationType, NotificationReason};
pub use channels::{Channel, ChannelConfig};
pub use email::{EmailMessage, EmailRenderer};
pub use service::{NotificationService, NotificationEvent};
