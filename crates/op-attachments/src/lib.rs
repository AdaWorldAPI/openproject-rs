//! # op-attachments
//!
//! File attachment handling for OpenProject RS.
//!
//! ## Features
//!
//! - Storage abstraction (local filesystem, S3-compatible)
//! - Attachment metadata management
//! - File upload and download
//! - Container associations (work packages, wiki pages, etc.)
//!
//! ## Example
//!
//! ```rust,ignore
//! use op_attachments::{AttachmentService, MemoryAttachmentStore, MemoryStorage};
//! use std::sync::Arc;
//!
//! let store = Arc::new(MemoryAttachmentStore::new());
//! let storage = Arc::new(MemoryStorage::new());
//! let service = AttachmentService::new(store, storage, Default::default());
//!
//! // Upload a file
//! let attachment = service.create(
//!     CreateAttachmentParams::new("document.pdf"),
//!     bytes::Bytes::from(file_data),
//!     user_id,
//! ).await?;
//! ```

pub mod model;
pub mod service;
pub mod storage;

pub use model::{
    Attachment, AttachmentThumbnail, AttachmentWithUrl, ContainerType, CreateAttachmentParams,
    ImageDimensions, ThumbnailSize,
};
pub use service::{
    AllowedFileTypes, AttachmentConfig, AttachmentError, AttachmentResult, AttachmentService,
    AttachmentStore, MemoryAttachmentStore,
};
pub use storage::{
    generate_disk_filename, generate_key, FileMetadata, LocalStorage, MemoryStorage, S3Config,
    S3Storage, Storage, StorageError, StorageResult,
};
