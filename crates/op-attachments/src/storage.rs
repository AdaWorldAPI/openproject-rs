//! Storage Abstraction
//!
//! Provides a unified interface for file storage backends.

use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Storage limit exceeded")]
    LimitExceeded,
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Storage backend error: {0}")]
    BackendError(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

/// File metadata from storage
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    /// Content type (MIME)
    pub content_type: String,
    /// SHA256 digest
    pub digest: String,
    /// Last modified time
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

/// Storage trait - unified interface for storage backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store data with a key
    async fn put(&self, key: &str, data: Bytes) -> StorageResult<FileMetadata>;

    /// Retrieve data by key
    async fn get(&self, key: &str) -> StorageResult<Bytes>;

    /// Delete data by key
    async fn delete(&self, key: &str) -> StorageResult<()>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> StorageResult<bool>;

    /// Get file metadata
    async fn metadata(&self, key: &str) -> StorageResult<FileMetadata>;

    /// Get a presigned URL for direct access (if supported)
    async fn url(&self, key: &str, expires_in: Duration) -> StorageResult<String>;

    /// Copy a file to a new key
    async fn copy(&self, from_key: &str, to_key: &str) -> StorageResult<()>;

    /// Get storage name for logging
    fn name(&self) -> &str;
}

/// Local filesystem storage
pub struct LocalStorage {
    /// Root directory for storage
    root: PathBuf,
    /// Base URL for generating URLs
    base_url: String,
}

impl LocalStorage {
    /// Create a new local storage
    pub fn new(root: impl AsRef<Path>, base_url: impl Into<String>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            base_url: base_url.into(),
        }
    }

    /// Create storage with a temp directory
    pub fn temp() -> std::io::Result<Self> {
        let dir = std::env::temp_dir().join("openproject-attachments");
        std::fs::create_dir_all(&dir)?;
        Ok(Self::new(dir, "/attachments"))
    }

    /// Resolve a key to a full path
    fn resolve_path(&self, key: &str) -> StorageResult<PathBuf> {
        // Prevent directory traversal
        if key.contains("..") || key.starts_with('/') || key.starts_with('\\') {
            return Err(StorageError::InvalidPath(key.to_string()));
        }

        Ok(self.root.join(key))
    }

    /// Ensure parent directory exists
    async fn ensure_parent(&self, path: &Path) -> StorageResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    /// Calculate SHA256 digest
    fn calculate_digest(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Guess content type from filename
    fn guess_content_type(key: &str) -> String {
        mime_guess::from_path(key)
            .first_or_octet_stream()
            .to_string()
    }
}

#[async_trait]
impl Storage for LocalStorage {
    #[instrument(skip(self, data), fields(storage = "local"))]
    async fn put(&self, key: &str, data: Bytes) -> StorageResult<FileMetadata> {
        let path = self.resolve_path(key)?;
        self.ensure_parent(&path).await?;

        let digest = Self::calculate_digest(&data);
        let size = data.len() as u64;
        let content_type = Self::guess_content_type(key);

        let mut file = fs::File::create(&path).await?;
        file.write_all(&data).await?;
        file.sync_all().await?;

        debug!(path = ?path, size = size, "File stored");

        Ok(FileMetadata {
            size,
            content_type,
            digest,
            last_modified: Some(chrono::Utc::now()),
        })
    }

    #[instrument(skip(self), fields(storage = "local"))]
    async fn get(&self, key: &str) -> StorageResult<Bytes> {
        let path = self.resolve_path(key)?;

        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        let mut file = fs::File::open(&path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        Ok(Bytes::from(buffer))
    }

    #[instrument(skip(self), fields(storage = "local"))]
    async fn delete(&self, key: &str) -> StorageResult<()> {
        let path = self.resolve_path(key)?;

        if path.exists() {
            fs::remove_file(&path).await?;
            debug!(path = ?path, "File deleted");
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let path = self.resolve_path(key)?;
        Ok(path.exists())
    }

    async fn metadata(&self, key: &str) -> StorageResult<FileMetadata> {
        let path = self.resolve_path(key)?;

        if !path.exists() {
            return Err(StorageError::NotFound(key.to_string()));
        }

        let meta = fs::metadata(&path).await?;
        let data = self.get(key).await?;
        let digest = Self::calculate_digest(&data);

        Ok(FileMetadata {
            size: meta.len(),
            content_type: Self::guess_content_type(key),
            digest,
            last_modified: meta
                .modified()
                .ok()
                .map(|t| chrono::DateTime::from(t)),
        })
    }

    async fn url(&self, key: &str, _expires_in: Duration) -> StorageResult<String> {
        // Local storage uses direct URLs
        Ok(format!("{}/{}", self.base_url, key))
    }

    async fn copy(&self, from_key: &str, to_key: &str) -> StorageResult<()> {
        let from_path = self.resolve_path(from_key)?;
        let to_path = self.resolve_path(to_key)?;

        if !from_path.exists() {
            return Err(StorageError::NotFound(from_key.to_string()));
        }

        self.ensure_parent(&to_path).await?;
        fs::copy(&from_path, &to_path).await?;

        Ok(())
    }

    fn name(&self) -> &str {
        "local"
    }
}

/// In-memory storage for testing
pub struct MemoryStorage {
    files: tokio::sync::RwLock<std::collections::HashMap<String, (Bytes, FileMetadata)>>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            files: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn put(&self, key: &str, data: Bytes) -> StorageResult<FileMetadata> {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let digest = hex::encode(hasher.finalize());

        let metadata = FileMetadata {
            size: data.len() as u64,
            content_type: mime_guess::from_path(key)
                .first_or_octet_stream()
                .to_string(),
            digest,
            last_modified: Some(chrono::Utc::now()),
        };

        let mut files = self.files.write().await;
        files.insert(key.to_string(), (data, metadata.clone()));

        Ok(metadata)
    }

    async fn get(&self, key: &str) -> StorageResult<Bytes> {
        let files = self.files.read().await;
        files
            .get(key)
            .map(|(data, _)| data.clone())
            .ok_or_else(|| StorageError::NotFound(key.to_string()))
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let mut files = self.files.write().await;
        files.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let files = self.files.read().await;
        Ok(files.contains_key(key))
    }

    async fn metadata(&self, key: &str) -> StorageResult<FileMetadata> {
        let files = self.files.read().await;
        files
            .get(key)
            .map(|(_, meta)| meta.clone())
            .ok_or_else(|| StorageError::NotFound(key.to_string()))
    }

    async fn url(&self, key: &str, _expires_in: Duration) -> StorageResult<String> {
        Ok(format!("/memory/{}", key))
    }

    async fn copy(&self, from_key: &str, to_key: &str) -> StorageResult<()> {
        let files = self.files.read().await;
        let (data, meta) = files
            .get(from_key)
            .ok_or_else(|| StorageError::NotFound(from_key.to_string()))?;
        let data = data.clone();
        let meta = meta.clone();
        drop(files);

        let mut files = self.files.write().await;
        files.insert(to_key.to_string(), (data, meta));
        Ok(())
    }

    fn name(&self) -> &str {
        "memory"
    }
}

/// S3-compatible storage configuration
#[derive(Debug, Clone)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub path_style: bool,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: "openproject-attachments".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
            access_key_id: String::new(),
            secret_access_key: String::new(),
            path_style: false,
        }
    }
}

/// S3-compatible storage (stub - requires aws-sdk-s3)
pub struct S3Storage {
    config: S3Config,
    // In a real implementation, this would use aws-sdk-s3
}

impl S3Storage {
    pub fn new(config: S3Config) -> Self {
        info!(bucket = %config.bucket, region = %config.region, "S3 storage initialized");
        Self { config }
    }

    fn key_url(&self, key: &str) -> String {
        if let Some(ref endpoint) = self.config.endpoint {
            if self.config.path_style {
                format!("{}/{}/{}", endpoint, self.config.bucket, key)
            } else {
                format!(
                    "{}.{}/{}",
                    self.config.bucket,
                    endpoint.trim_start_matches("https://").trim_start_matches("http://"),
                    key
                )
            }
        } else {
            format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.config.bucket, self.config.region, key
            )
        }
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn put(&self, key: &str, data: Bytes) -> StorageResult<FileMetadata> {
        // Stub implementation - would use aws-sdk-s3 in production
        error!("S3 storage not fully implemented");
        Err(StorageError::BackendError("S3 not implemented".to_string()))
    }

    async fn get(&self, key: &str) -> StorageResult<Bytes> {
        error!("S3 storage not fully implemented");
        Err(StorageError::BackendError("S3 not implemented".to_string()))
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        error!("S3 storage not fully implemented");
        Err(StorageError::BackendError("S3 not implemented".to_string()))
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        error!("S3 storage not fully implemented");
        Err(StorageError::BackendError("S3 not implemented".to_string()))
    }

    async fn metadata(&self, key: &str) -> StorageResult<FileMetadata> {
        error!("S3 storage not fully implemented");
        Err(StorageError::BackendError("S3 not implemented".to_string()))
    }

    async fn url(&self, key: &str, _expires_in: Duration) -> StorageResult<String> {
        // Return unsigned URL - real implementation would use presigned URLs
        Ok(self.key_url(key))
    }

    async fn copy(&self, from_key: &str, to_key: &str) -> StorageResult<()> {
        error!("S3 storage not fully implemented");
        Err(StorageError::BackendError("S3 not implemented".to_string()))
    }

    fn name(&self) -> &str {
        "s3"
    }
}

/// Generate a unique storage key
pub fn generate_key(filename: &str) -> String {
    let uuid = Uuid::new_v4();
    let date = chrono::Utc::now().format("%Y/%m/%d");
    format!("{}/{}/{}", date, uuid, filename)
}

/// Generate a disk filename (safe for filesystem)
pub fn generate_disk_filename(filename: &str) -> String {
    let uuid = Uuid::new_v4();
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if ext.is_empty() {
        format!("{}", uuid)
    } else {
        format!("{}.{}", uuid, ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_put_get() {
        let storage = MemoryStorage::new();
        let data = Bytes::from("Hello, World!");

        let meta = storage.put("test.txt", data.clone()).await.unwrap();
        assert_eq!(meta.size, 13);
        assert_eq!(meta.content_type, "text/plain");

        let retrieved = storage.get("test.txt").await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = MemoryStorage::new();
        let data = Bytes::from("test data");

        storage.put("test.txt", data).await.unwrap();
        assert!(storage.exists("test.txt").await.unwrap());

        storage.delete("test.txt").await.unwrap();
        assert!(!storage.exists("test.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_copy() {
        let storage = MemoryStorage::new();
        let data = Bytes::from("copy me");

        storage.put("original.txt", data.clone()).await.unwrap();
        storage.copy("original.txt", "copied.txt").await.unwrap();

        let copied = storage.get("copied.txt").await.unwrap();
        assert_eq!(copied, data);
    }

    #[tokio::test]
    async fn test_memory_storage_not_found() {
        let storage = MemoryStorage::new();

        let result = storage.get("nonexistent.txt").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[test]
    fn test_generate_key() {
        let key = generate_key("document.pdf");
        assert!(key.contains("document.pdf"));
        assert!(key.contains('/'));
    }

    #[test]
    fn test_generate_disk_filename() {
        let filename = generate_disk_filename("report.xlsx");
        assert!(filename.ends_with(".xlsx"));

        let no_ext = generate_disk_filename("noext");
        assert!(!no_ext.contains('.'));
    }

    #[tokio::test]
    async fn test_local_storage_path_traversal() {
        let storage = LocalStorage::temp().unwrap();

        let result = storage.get("../../../etc/passwd").await;
        assert!(matches!(result, Err(StorageError::InvalidPath(_))));
    }
}
