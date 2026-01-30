//! Common types used throughout OpenProject RS

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Formattable text (plain text or markdown with HTML output)
/// Mirrors OpenProject's Formattable concern
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Formattable {
    /// Raw content (markdown or plain text)
    pub raw: String,
    /// Rendered HTML content
    pub html: String,
    /// Format type (plain or markdown)
    pub format: TextFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TextFormat {
    #[default]
    Markdown,
    Plain,
}

impl Formattable {
    pub fn plain(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            raw: text.clone(),
            html: text,
            format: TextFormat::Plain,
        }
    }

    pub fn markdown(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        // TODO: Render markdown to HTML
        Self {
            html: raw.clone(), // Placeholder
            raw,
            format: TextFormat::Markdown,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}

/// Duration in ISO 8601 format (mirrors OpenProject's duration handling)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Duration {
    /// Duration in hours
    pub hours: f64,
    /// ISO 8601 duration string (e.g., "PT8H")
    pub iso8601: String,
}

impl Duration {
    pub fn from_hours(hours: f64) -> Self {
        let whole_hours = hours.floor() as i64;
        let minutes = ((hours - hours.floor()) * 60.0).round() as i64;

        let iso8601 = if minutes > 0 {
            format!("PT{}H{}M", whole_hours, minutes)
        } else {
            format!("PT{}H", whole_hours)
        };

        Self { hours, iso8601 }
    }

    pub fn from_iso8601(iso: &str) -> Option<Self> {
        // Basic ISO 8601 duration parsing
        // Full implementation would use a proper parser
        let hours = 0.0; // TODO: Parse properly
        Some(Self {
            hours,
            iso8601: iso.to_string(),
        })
    }
}

/// Date range (start_date to due_date)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DateRange {
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
}

impl DateRange {
    pub fn new(start: Option<NaiveDate>, due: Option<NaiveDate>) -> Self {
        Self {
            start_date: start,
            due_date: due,
        }
    }

    pub fn duration_days(&self) -> Option<i64> {
        match (self.start_date, self.due_date) {
            (Some(start), Some(due)) => Some((due - start).num_days()),
            _ => None,
        }
    }
}

/// Color representation (for types, statuses, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    /// Hex color code (e.g., "#FF0000")
    pub hex: String,
    /// Human-readable name
    pub name: Option<String>,
}

impl Color {
    pub fn new(hex: impl Into<String>) -> Self {
        Self {
            hex: hex.into(),
            name: None,
        }
    }

    pub fn with_name(hex: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            hex: hex.into(),
            name: Some(name.into()),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            hex: "#1A67A3".to_string(), // OpenProject blue
            name: None,
        }
    }
}

/// User status enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    #[default]
    Active,
    Registered,
    Locked,
    Invited,
}

impl UserStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn can_login(&self) -> bool {
        matches!(self, Self::Active | Self::Invited)
    }
}

/// Work package status (open/closed state)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatusCategory {
    Open,
    Closed,
}

/// Notification reason enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationReason {
    Mentioned,
    Assigned,
    Responsible,
    Watched,
    Subscribed,
    Commented,
    Created,
    Updated,
    Prioritized,
    Scheduled,
}

/// Attachment MIME type category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentCategory {
    Image,
    Video,
    Audio,
    Document,
    Spreadsheet,
    Presentation,
    Archive,
    Code,
    Other,
}

impl AttachmentCategory {
    pub fn from_mime_type(mime: &str) -> Self {
        if mime.starts_with("image/") {
            Self::Image
        } else if mime.starts_with("video/") {
            Self::Video
        } else if mime.starts_with("audio/") {
            Self::Audio
        } else if mime.contains("pdf")
            || mime.contains("document")
            || mime.contains("msword")
            || mime.contains("text/")
        {
            Self::Document
        } else if mime.contains("spreadsheet") || mime.contains("excel") {
            Self::Spreadsheet
        } else if mime.contains("presentation") || mime.contains("powerpoint") {
            Self::Presentation
        } else if mime.contains("zip") || mime.contains("tar") || mime.contains("archive") {
            Self::Archive
        } else if mime.contains("javascript")
            || mime.contains("json")
            || mime.contains("xml")
            || mime.contains("yaml")
        {
            Self::Code
        } else {
            Self::Other
        }
    }
}

/// Schedule mode for work packages
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleMode {
    #[default]
    Manual,
    Automatic,
}

/// Journal (audit log) entry types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JournalType {
    Created,
    Updated,
    Deleted,
    Comment,
}

/// HAL link representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalLink {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templated: Option<bool>,
}

impl HalLink {
    pub fn new(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            title: None,
            method: None,
            templated: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    pub fn templated(mut self) -> Self {
        self.templated = Some(true);
        self
    }
}
