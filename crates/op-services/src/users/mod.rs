//! User services
//!
//! Mirrors:
//! - app/services/users/create_service.rb
//! - app/services/users/update_service.rb
//! - app/services/users/delete_service.rb
//! - app/services/users/set_attributes_service.rb

mod create;
mod update;
mod delete;
mod set_attributes;

pub use create::CreateUserService;
pub use update::UpdateUserService;
pub use delete::DeleteUserService;
pub use set_attributes::{SetAttributesService, UserEntity};

/// Parameters for user operations
#[derive(Debug, Clone, Default)]
pub struct UserParams {
    pub login: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub mail: Option<String>,
    pub password: Option<String>,
    pub admin: Option<bool>,
    pub status: Option<i32>,
    pub language: Option<String>,
    pub force_password_change: Option<bool>,
    pub send_notifications: bool,
}

impl UserParams {
    pub fn new() -> Self {
        Self {
            send_notifications: true,
            ..Default::default()
        }
    }

    pub fn with_login(mut self, login: impl Into<String>) -> Self {
        self.login = Some(login.into());
        self
    }

    pub fn with_firstname(mut self, firstname: impl Into<String>) -> Self {
        self.firstname = Some(firstname.into());
        self
    }

    pub fn with_lastname(mut self, lastname: impl Into<String>) -> Self {
        self.lastname = Some(lastname.into());
        self
    }

    pub fn with_mail(mut self, mail: impl Into<String>) -> Self {
        self.mail = Some(mail.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_admin(mut self, admin: bool) -> Self {
        self.admin = Some(admin);
        self
    }

    pub fn with_status(mut self, status: i32) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn with_force_password_change(mut self, force: bool) -> Self {
        self.force_password_change = Some(force);
        self
    }

    pub fn without_notifications(mut self) -> Self {
        self.send_notifications = false;
        self
    }
}
