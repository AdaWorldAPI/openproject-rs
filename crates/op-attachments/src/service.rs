//! Attachment Service
//!
//! Orchestrates attachment operations, storage, and metadata management.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use op_core::traits::Id;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::model::{Attachment, AttachmentWithUrl, ContainerType, CreateAttachmentParams};
use crate::storage::{generate_disk_filename, Storage, StorageError};

/// Service errors
#[derive(Debug, Error)]
pub enum AttachmentError {
    #[error("Attachment not found: {0}")]
    NotFound(Id),
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    #[error("Invalid file: {0}")]
    InvalidFile(String),
    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: i64, max: i64 },
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Container not found: {0} {1}")]
    ContainerNotFound(String, Id),
}

pub type AttachmentResult<T> = Result<T, AttachmentError>;

/// Attachment store trait
#[async_trait]
pub trait AttachmentStore: Send + Sync {
    /// Create an attachment record
    async fn create(&self, attachment: &mut Attachment) -> AttachmentResult<Id>;

    /// Get an attachment by ID
    async fn get(&self, id: Id) -> AttachmentResult<Option<Attachment>>;

    /// Get attachments for a container
    async fn get_for_container(
        &self,
        container_type: ContainerType,
        container_id: Id,
    ) -> AttachmentResult<Vec<Attachment>>;

    /// Update an attachment
    async fn update(&self, attachment: &Attachment) -> AttachmentResult<()>;

    /// Delete an attachment
    async fn delete(&self, id: Id) -> AttachmentResult<()>;

    /// Get all orphaned attachments (no container)
    async fn get_orphaned(&self, older_than: chrono::DateTime<chrono::Utc>)
        -> AttachmentResult<Vec<Attachment>>;

    /// Count attachments for a container
    async fn count_for_container(
        &self,
        container_type: ContainerType,
        container_id: Id,
    ) -> AttachmentResult<usize>;
}

/// In-memory attachment store for testing
pub struct MemoryAttachmentStore {
    attachments: RwLock<Vec<Attachment>>,
    next_id: std::sync::atomic::AtomicI64,
}

impl Default for MemoryAttachmentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryAttachmentStore {
    pub fn new() -> Self {
        Self {
            attachments: RwLock::new(Vec::new()),
            next_id: std::sync::atomic::AtomicI64::new(1),
        }
    }
}

#[async_trait]
impl AttachmentStore for MemoryAttachmentStore {
    async fn create(&self, attachment: &mut Attachment) -> AttachmentResult<Id> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        attachment.id = Some(id);

        let mut attachments = self.attachments.write().await;
        attachments.push(attachment.clone());

        Ok(id)
    }

    async fn get(&self, id: Id) -> AttachmentResult<Option<Attachment>> {
        let attachments = self.attachments.read().await;
        Ok(attachments.iter().find(|a| a.id == Some(id)).cloned())
    }

    async fn get_for_container(
        &self,
        container_type: ContainerType,
        container_id: Id,
    ) -> AttachmentResult<Vec<Attachment>> {
        let attachments = self.attachments.read().await;
        Ok(attachments
            .iter()
            .filter(|a| {
                a.container_type == container_type.as_str()
                    && a.container_id == Some(container_id)
            })
            .cloned()
            .collect())
    }

    async fn update(&self, attachment: &Attachment) -> AttachmentResult<()> {
        let mut attachments = self.attachments.write().await;
        if let Some(pos) = attachments.iter().position(|a| a.id == attachment.id) {
            attachments[pos] = attachment.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: Id) -> AttachmentResult<()> {
        let mut attachments = self.attachments.write().await;
        attachments.retain(|a| a.id != Some(id));
        Ok(())
    }

    async fn get_orphaned(
        &self,
        older_than: chrono::DateTime<chrono::Utc>,
    ) -> AttachmentResult<Vec<Attachment>> {
        let attachments = self.attachments.read().await;
        Ok(attachments
            .iter()
            .filter(|a| a.container_id.is_none() && a.created_at < older_than)
            .cloned()
            .collect())
    }

    async fn count_for_container(
        &self,
        container_type: ContainerType,
        container_id: Id,
    ) -> AttachmentResult<usize> {
        let attachments = self.attachments.read().await;
        Ok(attachments
            .iter()
            .filter(|a| {
                a.container_type == container_type.as_str()
                    && a.container_id == Some(container_id)
            })
            .count())
    }
}

/// Allowed file types configuration
#[derive(Debug, Clone)]
pub struct AllowedFileTypes {
    /// Allowed MIME types (empty = allow all)
    pub allowed_mime_types: Vec<String>,
    /// Blocked MIME types
    pub blocked_mime_types: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: i64,
}

impl Default for AllowedFileTypes {
    fn default() -> Self {
        Self {
            allowed_mime_types: Vec::new(),
            blocked_mime_types: vec![
                "application/x-msdownload".to_string(),
                "application/x-executable".to_string(),
            ],
            max_file_size: 100 * 1024 * 1024, // 100 MB
        }
    }
}

impl AllowedFileTypes {
    /// Check if a content type is allowed
    pub fn is_allowed(&self, content_type: &str) -> bool {
        // Check blocked list first
        if self.blocked_mime_types.iter().any(|t| t == content_type) {
            return false;
        }

        // If allowed list is empty, allow all (except blocked)
        if self.allowed_mime_types.is_empty() {
            return true;
        }

        // Check allowed list
        self.allowed_mime_types.iter().any(|t| t == content_type)
    }
}

/// Attachment service configuration
#[derive(Debug, Clone)]
pub struct AttachmentConfig {
    pub allowed_types: AllowedFileTypes,
    pub url_expiry: Duration,
    pub cleanup_orphans_after: Duration,
}

impl Default for AttachmentConfig {
    fn default() -> Self {
        Self {
            allowed_types: AllowedFileTypes::default(),
            url_expiry: Duration::from_secs(3600), // 1 hour
            cleanup_orphans_after: Duration::from_secs(86400), // 24 hours
        }
    }
}

/// Attachment service
pub struct AttachmentService<St: AttachmentStore, S: Storage> {
    store: Arc<St>,
    storage: Arc<S>,
    config: AttachmentConfig,
}

impl<St: AttachmentStore, S: Storage> AttachmentService<St, S> {
    pub fn new(store: Arc<St>, storage: Arc<S>, config: AttachmentConfig) -> Self {
        Self {
            store,
            storage,
            config,
        }
    }

    /// Create an attachment from uploaded data
    #[instrument(skip(self, data, author_id), fields(filename = %params.filename))]
    pub async fn create(
        &self,
        params: CreateAttachmentParams,
        data: Bytes,
        author_id: Id,
    ) -> AttachmentResult<AttachmentWithUrl> {
        let size = data.len() as i64;

        // Check file size
        if size > self.config.allowed_types.max_file_size {
            return Err(AttachmentError::FileTooLarge {
                size,
                max: self.config.allowed_types.max_file_size,
            });
        }

        // Determine content type
        let content_type = params.content_type.clone().unwrap_or_else(|| {
            mime_guess::from_path(&params.filename)
                .first_or_octet_stream()
                .to_string()
        });

        // Check content type
        if !self.config.allowed_types.is_allowed(&content_type) {
            return Err(AttachmentError::InvalidContentType(content_type));
        }

        // Generate storage key
        let disk_filename = generate_disk_filename(&params.filename);

        // Store file
        let metadata = self.storage.put(&disk_filename, data).await?;

        // Create attachment record
        let mut attachment = Attachment::new(
            &params.filename,
            &disk_filename,
            metadata.size as i64,
            &content_type,
            &metadata.digest,
            author_id,
        );

        if let Some(desc) = params.description {
            attachment.description = Some(desc);
        }

        if let (Some(ct), Some(cid)) = (params.container_type, params.container_id) {
            attachment = attachment.for_container(ct, cid);
        }

        // Store record
        let id = self.store.create(&mut attachment).await?;
        info!(id = id, filename = %params.filename, "Attachment created");

        // Get download URL
        let download_url = self
            .storage
            .url(&disk_filename, self.config.url_expiry)
            .await?;

        Ok(AttachmentWithUrl::new(attachment, download_url))
    }

    /// Get an attachment by ID
    pub async fn get(&self, id: Id) -> AttachmentResult<Option<Attachment>> {
        self.store.get(id).await
    }

    /// Get an attachment with download URL
    pub async fn get_with_url(&self, id: Id) -> AttachmentResult<Option<AttachmentWithUrl>> {
        let attachment = self.store.get(id).await?;

        match attachment {
            Some(a) => {
                let url = self
                    .storage
                    .url(&a.disk_filename, self.config.url_expiry)
                    .await?;
                Ok(Some(AttachmentWithUrl::new(a, url)))
            }
            None => Ok(None),
        }
    }

    /// Get attachments for a container
    pub async fn get_for_container(
        &self,
        container_type: ContainerType,
        container_id: Id,
    ) -> AttachmentResult<Vec<Attachment>> {
        self.store.get_for_container(container_type, container_id).await
    }

    /// Download attachment data
    #[instrument(skip(self))]
    pub async fn download(&self, id: Id) -> AttachmentResult<(Attachment, Bytes)> {
        let attachment = self
            .store
            .get(id)
            .await?
            .ok_or(AttachmentError::NotFound(id))?;

        let data = self.storage.get(&attachment.disk_filename).await?;

        // Update download count
        let mut updated = attachment.clone();
        updated.increment_downloads();
        self.store.update(&updated).await?;

        debug!(id = id, downloads = updated.downloads, "Attachment downloaded");

        Ok((attachment, data))
    }

    /// Attach to a container
    pub async fn attach_to(
        &self,
        id: Id,
        container_type: ContainerType,
        container_id: Id,
    ) -> AttachmentResult<Attachment> {
        let mut attachment = self
            .store
            .get(id)
            .await?
            .ok_or(AttachmentError::NotFound(id))?;

        attachment.container_type = container_type.to_string();
        attachment.container_id = Some(container_id);
        attachment.updated_at = chrono::Utc::now();

        self.store.update(&attachment).await?;

        info!(
            id = id,
            container = %container_type,
            container_id = container_id,
            "Attachment attached to container"
        );

        Ok(attachment)
    }

    /// Delete an attachment
    #[instrument(skip(self))]
    pub async fn delete(&self, id: Id) -> AttachmentResult<()> {
        let attachment = self
            .store
            .get(id)
            .await?
            .ok_or(AttachmentError::NotFound(id))?;

        // Delete from storage
        self.storage.delete(&attachment.disk_filename).await?;

        // Delete record
        self.store.delete(id).await?;

        info!(id = id, filename = %attachment.filename, "Attachment deleted");

        Ok(())
    }

    /// Cleanup orphaned attachments
    #[instrument(skip(self))]
    pub async fn cleanup_orphans(&self) -> AttachmentResult<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(self.config.cleanup_orphans_after)
            .unwrap_or(chrono::Duration::days(1));

        let orphans = self.store.get_orphaned(cutoff).await?;
        let count = orphans.len();

        for attachment in orphans {
            if let Some(id) = attachment.id {
                if let Err(e) = self.delete(id).await {
                    warn!(id = id, error = %e, "Failed to delete orphaned attachment");
                }
            }
        }

        info!(count = count, "Orphaned attachments cleaned up");

        Ok(count)
    }

    /// Copy an attachment to a new container
    pub async fn copy_to(
        &self,
        id: Id,
        container_type: ContainerType,
        container_id: Id,
        author_id: Id,
    ) -> AttachmentResult<AttachmentWithUrl> {
        let source = self
            .store
            .get(id)
            .await?
            .ok_or(AttachmentError::NotFound(id))?;

        // Create new disk filename
        let new_disk_filename = generate_disk_filename(&source.filename);

        // Copy in storage
        self.storage
            .copy(&source.disk_filename, &new_disk_filename)
            .await?;

        // Create new attachment record
        let mut new_attachment = Attachment::new(
            &source.filename,
            &new_disk_filename,
            source.filesize,
            &source.content_type,
            &source.digest,
            author_id,
        )
        .for_container(container_type, container_id);

        new_attachment.description = source.description.clone();

        let new_id = self.store.create(&mut new_attachment).await?;

        info!(
            source_id = id,
            new_id = new_id,
            container = %container_type,
            "Attachment copied"
        );

        let url = self
            .storage
            .url(&new_disk_filename, self.config.url_expiry)
            .await?;

        Ok(AttachmentWithUrl::new(new_attachment, url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    fn create_service() -> AttachmentService<MemoryAttachmentStore, MemoryStorage> {
        let store = Arc::new(MemoryAttachmentStore::new());
        let storage = Arc::new(MemoryStorage::new());
        AttachmentService::new(store, storage, AttachmentConfig::default())
    }

    #[tokio::test]
    async fn test_create_attachment() {
        let service = create_service();
        let data = Bytes::from("Hello, World!");

        let result = service
            .create(CreateAttachmentParams::new("test.txt"), data, 1)
            .await
            .unwrap();

        assert_eq!(result.attachment.filename, "test.txt");
        assert_eq!(result.attachment.filesize, 13);
        assert_eq!(result.attachment.content_type, "text/plain");
        assert!(!result.download_url.is_empty());
    }

    #[tokio::test]
    async fn test_create_with_container() {
        let service = create_service();
        let data = Bytes::from("PDF content");

        let result = service
            .create(
                CreateAttachmentParams::new("document.pdf")
                    .container(ContainerType::WorkPackage, 100),
                data,
                1,
            )
            .await
            .unwrap();

        assert!(result.attachment.is_attached());
        assert_eq!(result.attachment.container_id, Some(100));
    }

    #[tokio::test]
    async fn test_download_increments_count() {
        let service = create_service();
        let data = Bytes::from("download me");

        let created = service
            .create(CreateAttachmentParams::new("file.txt"), data.clone(), 1)
            .await
            .unwrap();

        let id = created.attachment.id.unwrap();

        // Download and check count increases
        let (_, downloaded) = service.download(id).await.unwrap();
        assert_eq!(downloaded, data);

        let updated = service.get(id).await.unwrap().unwrap();
        assert_eq!(updated.downloads, 1);
    }

    #[tokio::test]
    async fn test_delete_attachment() {
        let service = create_service();
        let data = Bytes::from("delete me");

        let created = service
            .create(CreateAttachmentParams::new("temp.txt"), data, 1)
            .await
            .unwrap();

        let id = created.attachment.id.unwrap();

        service.delete(id).await.unwrap();

        let result = service.get(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_file_too_large() {
        let store = Arc::new(MemoryAttachmentStore::new());
        let storage = Arc::new(MemoryStorage::new());
        let mut config = AttachmentConfig::default();
        config.allowed_types.max_file_size = 10; // 10 bytes max

        let service = AttachmentService::new(store, storage, config);

        let data = Bytes::from("This is more than 10 bytes!");

        let result = service
            .create(CreateAttachmentParams::new("large.txt"), data, 1)
            .await;

        assert!(matches!(result, Err(AttachmentError::FileTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_blocked_content_type() {
        let service = create_service();
        let data = Bytes::from("fake executable");

        let result = service
            .create(
                CreateAttachmentParams::new("virus.exe")
                    .content_type("application/x-msdownload"),
                data,
                1,
            )
            .await;

        assert!(matches!(result, Err(AttachmentError::InvalidContentType(_))));
    }

    #[tokio::test]
    async fn test_attach_to_container() {
        let service = create_service();
        let data = Bytes::from("orphan file");

        let created = service
            .create(CreateAttachmentParams::new("orphan.txt"), data, 1)
            .await
            .unwrap();

        let id = created.attachment.id.unwrap();
        assert!(!created.attachment.is_attached());

        let attached = service
            .attach_to(id, ContainerType::WikiPage, 50)
            .await
            .unwrap();

        assert!(attached.is_attached());
        assert_eq!(attached.container_type, "WikiPage");
        assert_eq!(attached.container_id, Some(50));
    }

    #[tokio::test]
    async fn test_get_for_container() {
        let service = create_service();

        // Create attachments for different containers
        for i in 0..3 {
            service
                .create(
                    CreateAttachmentParams::new(format!("file{}.txt", i))
                        .container(ContainerType::WorkPackage, 100),
                    Bytes::from(format!("content {}", i)),
                    1,
                )
                .await
                .unwrap();
        }

        service
            .create(
                CreateAttachmentParams::new("other.txt")
                    .container(ContainerType::WorkPackage, 200),
                Bytes::from("other"),
                1,
            )
            .await
            .unwrap();

        let attachments = service
            .get_for_container(ContainerType::WorkPackage, 100)
            .await
            .unwrap();

        assert_eq!(attachments.len(), 3);
    }

    #[tokio::test]
    async fn test_copy_attachment() {
        let service = create_service();
        let data = Bytes::from("copy this");

        let created = service
            .create(
                CreateAttachmentParams::new("original.txt")
                    .container(ContainerType::WorkPackage, 100),
                data,
                1,
            )
            .await
            .unwrap();

        let id = created.attachment.id.unwrap();

        let copied = service
            .copy_to(id, ContainerType::WikiPage, 50, 2)
            .await
            .unwrap();

        assert_ne!(copied.attachment.id, created.attachment.id);
        assert_eq!(copied.attachment.filename, "original.txt");
        assert_eq!(copied.attachment.container_type, "WikiPage");
        assert_eq!(copied.attachment.container_id, Some(50));
        assert_eq!(copied.attachment.author_id, 2);
    }

    #[test]
    fn test_allowed_file_types() {
        let allowed = AllowedFileTypes::default();

        assert!(allowed.is_allowed("image/png"));
        assert!(allowed.is_allowed("application/pdf"));
        assert!(!allowed.is_allowed("application/x-msdownload"));
    }

    #[tokio::test]
    async fn test_memory_store_count() {
        let store = MemoryAttachmentStore::new();

        for i in 0..5 {
            let mut attachment = Attachment::new(
                format!("file{}.txt", i),
                format!("disk{}", i),
                100,
                "text/plain",
                "hash",
                1,
            )
            .for_container(ContainerType::Document, 10);
            store.create(&mut attachment).await.unwrap();
        }

        let count = store
            .count_for_container(ContainerType::Document, 10)
            .await
            .unwrap();
        assert_eq!(count, 5);
    }
}
