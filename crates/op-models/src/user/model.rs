//! User model
//!
//! Mirrors: app/models/user.rb
//! Table: users

use chrono::{DateTime, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped};
use op_core::types::UserStatus;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// User entity
///
/// Represents a user account in OpenProject.
/// Inherits from Principal (STI pattern in Ruby).
///
/// # Ruby equivalent
/// ```ruby
/// class User < Principal
///   has_many :members
///   has_many :projects, through: :members
///   has_many :assigned_work_packages
///   has_many :responsible_work_packages
///   # ... etc
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Option<Id>,

    /// Login name (unique)
    #[validate(length(min = 1, max = 255))]
    pub login: String,

    /// First name
    #[validate(length(max = 255))]
    pub firstname: String,

    /// Last name
    #[validate(length(max = 255))]
    pub lastname: String,

    /// Email address
    #[validate(email)]
    pub mail: String,

    /// Whether user is admin
    pub admin: bool,

    /// User status (active, locked, registered, invited)
    pub status: UserStatus,

    /// Hashed password
    #[serde(skip_serializing)]
    pub hashed_password: Option<String>,

    /// Password salt (legacy, not used with argon2)
    #[serde(skip)]
    pub salt: Option<String>,

    /// Force password change on next login
    pub force_password_change: bool,

    /// Failed login attempt count
    pub failed_login_count: i32,

    /// Last failed login timestamp
    pub last_failed_login_on: Option<DateTime<Utc>>,

    /// Last successful login timestamp
    pub last_login_on: Option<DateTime<Utc>>,

    /// Language preference
    pub language: Option<String>,

    /// Authentication source ID (for LDAP)
    pub auth_source_id: Option<Id>,

    /// Identity URL (for OpenID)
    pub identity_url: Option<String>,

    /// Notification email address (if different from mail)
    pub mail_notification: String,

    /// Notification settings as JSON
    pub notification_settings: Option<serde_json::Value>,

    /// User type (for STI - User, Group, PlaceholderUser)
    #[serde(rename = "type")]
    pub principal_type: String,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: None,
            login: String::new(),
            firstname: String::new(),
            lastname: String::new(),
            mail: String::new(),
            admin: false,
            status: UserStatus::Active,
            hashed_password: None,
            salt: None,
            force_password_change: false,
            failed_login_count: 0,
            last_failed_login_on: None,
            last_login_on: None,
            language: None,
            auth_source_id: None,
            identity_url: None,
            mail_notification: "only_my_events".to_string(),
            notification_settings: None,
            principal_type: "User".to_string(),
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for User {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for User {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl Entity for User {
    const TABLE_NAME: &'static str = "users";
    const TYPE_NAME: &'static str = "User";
}

impl User {
    /// Get full name (firstname + lastname)
    pub fn name(&self) -> String {
        format!("{} {}", self.firstname, self.lastname).trim().to_string()
    }

    /// Check if user is active and can log in
    pub fn active(&self) -> bool {
        self.status.is_active()
    }

    /// Check if user is locked
    pub fn locked(&self) -> bool {
        matches!(self.status, UserStatus::Locked)
    }

    /// Check if user can log in (not locked, not anonymous)
    pub fn can_login(&self) -> bool {
        self.status.can_login() && !self.login.is_empty()
    }

    /// Check if account is temporarily locked due to failed logins
    pub fn temporarily_locked(&self) -> bool {
        // 5 failed attempts within 30 minutes = locked
        if self.failed_login_count >= 5 {
            if let Some(last_failed) = self.last_failed_login_on {
                let threshold = Utc::now() - chrono::Duration::minutes(30);
                return last_failed > threshold;
            }
        }
        false
    }

    /// Register a failed login attempt
    pub fn register_failed_login(&mut self) {
        self.failed_login_count += 1;
        self.last_failed_login_on = Some(Utc::now());
    }

    /// Clear failed login attempts (on successful login)
    pub fn clear_failed_logins(&mut self) {
        self.failed_login_count = 0;
        self.last_failed_login_on = None;
        self.last_login_on = Some(Utc::now());
    }

    /// Builtin system users
    pub fn anonymous_id() -> Id {
        1 // Convention: anonymous user has ID 1
    }

    pub fn system_id() -> Id {
        0 // Convention: system user has ID 0
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self.id, Some(id) if id <= 1)
    }

    pub fn is_anonymous(&self) -> bool {
        self.id == Some(Self::anonymous_id())
    }
}

/// New user creation parameters
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 1, max = 255))]
    pub login: String,

    #[validate(length(max = 255))]
    pub firstname: String,

    #[validate(length(max = 255))]
    pub lastname: String,

    #[validate(email)]
    pub mail: String,

    #[validate(length(min = 10))]
    pub password: Option<String>,

    pub admin: Option<bool>,
    pub status: Option<UserStatus>,
    pub language: Option<String>,
    pub force_password_change: Option<bool>,
    pub auth_source_id: Option<Id>,
}

impl From<NewUser> for User {
    fn from(new: NewUser) -> Self {
        Self {
            login: new.login,
            firstname: new.firstname,
            lastname: new.lastname,
            mail: new.mail,
            admin: new.admin.unwrap_or(false),
            status: new.status.unwrap_or(UserStatus::Active),
            language: new.language,
            force_password_change: new.force_password_change.unwrap_or(true),
            auth_source_id: new.auth_source_id,
            ..Default::default()
        }
    }
}

/// User update parameters
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateUser {
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub mail: Option<String>,
    pub admin: Option<bool>,
    pub status: Option<UserStatus>,
    pub language: Option<String>,
    pub password: Option<String>,
    pub force_password_change: Option<bool>,
}
