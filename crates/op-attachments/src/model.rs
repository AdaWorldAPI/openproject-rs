//! Attachment Model
//!
//! Mirrors: app/models/attachment.rb

use chrono::{DateTime, Utc};
use op_core::traits::Id;
use serde::{Deserialize, Serialize};

/// Container types that can have attachments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ContainerType {
    WorkPackage,
    WikiPage,
    Document,
    Message,
    News,
    Project,
    Meeting,
    MeetingContent,
    Version,
    User,
}

impl ContainerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkPackage => "WorkPackage",
            Self::WikiPage => "WikiPage",
            Self::Document => "Document",
            Self::Message => "Message",
            Self::News => "News",
            Self::Project => "Project",
            Self::Meeting => "Meeting",
            Self::MeetingContent => "MeetingContent",
            Self::Version => "Version",
            Self::User => "User",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "WorkPackage" => Some(Self::WorkPackage),
            "WikiPage" => Some(Self::WikiPage),
            "Document" => Some(Self::Document),
            "Message" => Some(Self::Message),
            "News" => Some(Self::News),
            "Project" => Some(Self::Project),
            "Meeting" => Some(Self::Meeting),
            "MeetingContent" => Some(Self::MeetingContent),
            "Version" => Some(Self::Version),
            "User" => Some(Self::User),
            _ => None,
        }
    }
}

impl std::fmt::Display for ContainerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An attachment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID
    pub id: Option<Id>,
    /// Container type (e.g., "WorkPackage", "WikiPage")
    pub container_type: String,
    /// Container ID
    pub container_id: Option<Id>,
    /// Original filename
    pub filename: String,
    /// Filename on disk/storage
    pub disk_filename: String,
    /// File size in bytes
    pub filesize: i64,
    /// MIME content type
    pub content_type: String,
    /// SHA256 digest
    pub digest: String,
    /// Number of downloads
    pub downloads: i32,
    /// Author user ID
    pub author_id: Id,
    /// Description/caption
    pub description: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// File token for direct access
    pub file_token: Option<String>,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(
        filename: impl Into<String>,
        disk_filename: impl Into<String>,
        filesize: i64,
        content_type: impl Into<String>,
        digest: impl Into<String>,
        author_id: Id,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            container_type: String::new(),
            container_id: None,
            filename: filename.into(),
            disk_filename: disk_filename.into(),
            filesize,
            content_type: content_type.into(),
            digest: digest.into(),
            downloads: 0,
            author_id,
            description: None,
            created_at: now,
            updated_at: now,
            file_token: None,
        }
    }

    /// Set the container
    pub fn for_container(mut self, container_type: ContainerType, container_id: Id) -> Self {
        self.container_type = container_type.to_string();
        self.container_id = Some(container_id);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if this is an image
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    /// Check if this is a PDF
    pub fn is_pdf(&self) -> bool {
        self.content_type == "application/pdf"
    }

    /// Check if this has a container
    pub fn is_attached(&self) -> bool {
        self.container_id.is_some()
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        // Only return extension if there's a dot in the filename
        if !self.filename.contains('.') {
            return None;
        }
        self.filename
            .rsplit('.')
            .next()
            .filter(|ext| ext.len() <= 10 && !ext.is_empty())
    }

    /// Increment download count
    pub fn increment_downloads(&mut self) {
        self.downloads += 1;
        self.updated_at = Utc::now();
    }

    /// Human-readable file size
    pub fn human_filesize(&self) -> String {
        let size = self.filesize as f64;
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

        if size == 0.0 {
            return "0 B".to_string();
        }

        let base = 1024.0_f64;
        let i = (size.ln() / base.ln()).floor() as usize;
        let i = i.min(UNITS.len() - 1);

        let value = size / base.powi(i as i32);
        format!("{:.1} {}", value, UNITS[i])
    }
}

/// Parameters for creating an attachment
#[derive(Debug, Clone)]
pub struct CreateAttachmentParams {
    pub filename: String,
    pub content_type: Option<String>,
    pub description: Option<String>,
    pub container_type: Option<ContainerType>,
    pub container_id: Option<Id>,
}

impl CreateAttachmentParams {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            content_type: None,
            description: None,
            container_type: None,
            container_id: None,
        }
    }

    pub fn content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn container(mut self, ct: ContainerType, id: Id) -> Self {
        self.container_type = Some(ct);
        self.container_id = Some(id);
        self
    }
}

/// Attachment with embedded download URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentWithUrl {
    #[serde(flatten)]
    pub attachment: Attachment,
    pub download_url: String,
}

impl AttachmentWithUrl {
    pub fn new(attachment: Attachment, download_url: String) -> Self {
        Self {
            attachment,
            download_url,
        }
    }
}

/// Image dimensions (for image attachments)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// Attachment thumbnail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentThumbnail {
    pub attachment_id: Id,
    pub size: ThumbnailSize,
    pub disk_filename: String,
    pub dimensions: ImageDimensions,
}

/// Thumbnail sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThumbnailSize {
    Small,   // 64x64
    Medium,  // 128x128
    Large,   // 256x256
    Preview, // 800x600
}

impl ThumbnailSize {
    pub fn max_dimension(&self) -> u32 {
        match self {
            Self::Small => 64,
            Self::Medium => 128,
            Self::Large => 256,
            Self::Preview => 800,
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            Self::Small => "_small",
            Self::Medium => "_medium",
            Self::Large => "_large",
            Self::Preview => "_preview",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_creation() {
        let attachment = Attachment::new(
            "document.pdf",
            "abc123.pdf",
            1024 * 1024,
            "application/pdf",
            "sha256hash",
            1,
        );

        assert_eq!(attachment.filename, "document.pdf");
        assert!(attachment.is_pdf());
        assert!(!attachment.is_image());
        assert!(!attachment.is_attached());
    }

    #[test]
    fn test_attachment_with_container() {
        let attachment = Attachment::new(
            "image.png",
            "def456.png",
            2048,
            "image/png",
            "sha256hash",
            1,
        )
        .for_container(ContainerType::WorkPackage, 100);

        assert!(attachment.is_attached());
        assert!(attachment.is_image());
        assert_eq!(attachment.container_type, "WorkPackage");
        assert_eq!(attachment.container_id, Some(100));
    }

    #[test]
    fn test_human_filesize() {
        let cases = [
            (0, "0 B"),
            (512, "512.0 B"),
            (1024, "1.0 KB"),
            (1536, "1.5 KB"),
            (1024 * 1024, "1.0 MB"),
            (1024 * 1024 * 1024, "1.0 GB"),
        ];

        for (size, expected) in cases {
            let attachment = Attachment::new("test", "test", size, "text/plain", "hash", 1);
            assert_eq!(attachment.human_filesize(), expected, "Size: {}", size);
        }
    }

    #[test]
    fn test_extension() {
        let pdf = Attachment::new("report.pdf", "disk", 100, "application/pdf", "hash", 1);
        assert_eq!(pdf.extension(), Some("pdf"));

        let no_ext = Attachment::new("noextension", "disk", 100, "text/plain", "hash", 1);
        assert_eq!(no_ext.extension(), None); // No dot = no extension

        let double = Attachment::new("archive.tar.gz", "disk", 100, "application/gzip", "hash", 1);
        assert_eq!(double.extension(), Some("gz"));
    }

    #[test]
    fn test_container_type_conversion() {
        assert_eq!(ContainerType::WorkPackage.as_str(), "WorkPackage");
        assert_eq!(
            ContainerType::from_str("WikiPage"),
            Some(ContainerType::WikiPage)
        );
        assert_eq!(ContainerType::from_str("Unknown"), None);
    }

    #[test]
    fn test_thumbnail_sizes() {
        assert_eq!(ThumbnailSize::Small.max_dimension(), 64);
        assert_eq!(ThumbnailSize::Medium.max_dimension(), 128);
        assert_eq!(ThumbnailSize::Large.max_dimension(), 256);
        assert_eq!(ThumbnailSize::Preview.max_dimension(), 800);
    }

    #[test]
    fn test_increment_downloads() {
        let mut attachment =
            Attachment::new("test.txt", "disk", 100, "text/plain", "hash", 1);
        assert_eq!(attachment.downloads, 0);

        attachment.increment_downloads();
        assert_eq!(attachment.downloads, 1);

        attachment.increment_downloads();
        assert_eq!(attachment.downloads, 2);
    }
}
