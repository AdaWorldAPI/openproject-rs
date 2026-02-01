//! Background Job Queue
//!
//! Mirrors: app/workers/*.rb (Sidekiq jobs)

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

/// Job errors
#[derive(Debug, Error)]
pub enum JobError {
    #[error("Job not found: {0}")]
    NotFound(String),
    #[error("Job failed: {0}")]
    Failed(String),
    #[error("Retry limit exceeded")]
    RetryLimitExceeded,
    #[error("Queue error: {0}")]
    QueueError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type JobResult<T> = Result<T, JobError>;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
    Dead,
}

/// Job priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// A background job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job ID
    pub id: String,
    /// Job type (worker class name)
    pub job_type: String,
    /// Queue name
    pub queue: String,
    /// Job arguments (JSON)
    pub args: serde_json::Value,
    /// Current status
    pub status: JobStatus,
    /// Priority
    pub priority: JobPriority,
    /// Number of retry attempts
    pub retries: u32,
    /// Maximum retries allowed
    pub max_retries: u32,
    /// Error message (if failed)
    pub error: Option<String>,
    /// When to run (for scheduled jobs)
    pub run_at: Option<DateTime<Utc>>,
    /// When the job was created
    pub created_at: DateTime<Utc>,
    /// When the job started running
    pub started_at: Option<DateTime<Utc>>,
    /// When the job completed/failed
    pub finished_at: Option<DateTime<Utc>>,
}

impl Job {
    /// Create a new job
    pub fn new(job_type: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_type: job_type.into(),
            queue: "default".to_string(),
            args,
            status: JobStatus::Pending,
            priority: JobPriority::Normal,
            retries: 0,
            max_retries: 3,
            error: None,
            run_at: None,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
        }
    }

    /// Set the queue
    pub fn queue(mut self, queue: impl Into<String>) -> Self {
        self.queue = queue.into();
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Schedule for later
    pub fn run_at(mut self, at: DateTime<Utc>) -> Self {
        self.run_at = Some(at);
        self
    }

    /// Schedule in N seconds
    pub fn run_in(mut self, seconds: i64) -> Self {
        self.run_at = Some(Utc::now() + chrono::Duration::seconds(seconds));
        self
    }

    /// Check if the job is ready to run
    pub fn is_ready(&self) -> bool {
        match self.run_at {
            Some(at) => Utc::now() >= at,
            None => true,
        }
    }

    /// Check if the job can be retried
    pub fn can_retry(&self) -> bool {
        self.retries < self.max_retries
    }

    /// Mark as running
    pub fn mark_running(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark as completed
    pub fn mark_completed(&mut self) {
        self.status = JobStatus::Completed;
        self.finished_at = Some(Utc::now());
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
        self.finished_at = Some(Utc::now());

        if self.can_retry() {
            self.status = JobStatus::Retrying;
            self.retries += 1;
            // Exponential backoff: 2^retries minutes
            let delay = 2_i64.pow(self.retries) * 60;
            self.run_at = Some(Utc::now() + chrono::Duration::seconds(delay));
        } else {
            self.status = JobStatus::Dead;
        }
    }
}

/// Job queue trait
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Enqueue a job
    async fn enqueue(&self, job: Job) -> JobResult<String>;

    /// Get a job by ID
    async fn get(&self, job_id: &str) -> JobResult<Option<Job>>;

    /// Dequeue the next ready job
    async fn dequeue(&self, queue: &str) -> JobResult<Option<Job>>;

    /// Update a job
    async fn update(&self, job: &Job) -> JobResult<()>;

    /// Delete a job
    async fn delete(&self, job_id: &str) -> JobResult<()>;

    /// Get pending job count
    async fn pending_count(&self, queue: &str) -> JobResult<usize>;

    /// Get all jobs for a queue
    async fn list(&self, queue: &str, status: Option<JobStatus>) -> JobResult<Vec<Job>>;

    /// Retry all dead jobs
    async fn retry_dead(&self, queue: &str) -> JobResult<usize>;

    /// Clear completed jobs
    async fn clear_completed(&self, queue: &str) -> JobResult<usize>;
}

/// In-memory job queue for development/testing
pub struct MemoryJobQueue {
    jobs: RwLock<HashMap<String, Job>>,
    counter: AtomicU64,
}

impl Default for MemoryJobQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryJobQueue {
    pub fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
            counter: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl JobQueue for MemoryJobQueue {
    async fn enqueue(&self, mut job: Job) -> JobResult<String> {
        let mut jobs = self.jobs.write().await;
        let id = job.id.clone();
        jobs.insert(id.clone(), job);
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(id)
    }

    async fn get(&self, job_id: &str) -> JobResult<Option<Job>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.get(job_id).cloned())
    }

    async fn dequeue(&self, queue: &str) -> JobResult<Option<Job>> {
        let mut jobs = self.jobs.write().await;

        // Find the highest priority ready job
        let job_id = jobs
            .values()
            .filter(|j| {
                j.queue == queue
                    && j.status == JobStatus::Pending
                    && j.is_ready()
            })
            .max_by_key(|j| j.priority)
            .map(|j| j.id.clone());

        if let Some(id) = job_id {
            if let Some(job) = jobs.get_mut(&id) {
                job.mark_running();
                return Ok(Some(job.clone()));
            }
        }

        // Check for retrying jobs
        let retry_id = jobs
            .values()
            .filter(|j| {
                j.queue == queue
                    && j.status == JobStatus::Retrying
                    && j.is_ready()
            })
            .map(|j| j.id.clone())
            .next();

        if let Some(id) = retry_id {
            if let Some(job) = jobs.get_mut(&id) {
                job.status = JobStatus::Pending;
                job.mark_running();
                return Ok(Some(job.clone()));
            }
        }

        Ok(None)
    }

    async fn update(&self, job: &Job) -> JobResult<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job.clone());
        Ok(())
    }

    async fn delete(&self, job_id: &str) -> JobResult<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(job_id);
        Ok(())
    }

    async fn pending_count(&self, queue: &str) -> JobResult<usize> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .filter(|j| j.queue == queue && j.status == JobStatus::Pending)
            .count())
    }

    async fn list(&self, queue: &str, status: Option<JobStatus>) -> JobResult<Vec<Job>> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .filter(|j| {
                j.queue == queue && status.map_or(true, |s| j.status == s)
            })
            .cloned()
            .collect())
    }

    async fn retry_dead(&self, queue: &str) -> JobResult<usize> {
        let mut jobs = self.jobs.write().await;
        let mut count = 0;

        for job in jobs.values_mut() {
            if job.queue == queue && job.status == JobStatus::Dead {
                job.status = JobStatus::Pending;
                job.retries = 0;
                job.error = None;
                job.run_at = None;
                count += 1;
            }
        }

        Ok(count)
    }

    async fn clear_completed(&self, queue: &str) -> JobResult<usize> {
        let mut jobs = self.jobs.write().await;
        let to_remove: Vec<String> = jobs
            .values()
            .filter(|j| j.queue == queue && j.status == JobStatus::Completed)
            .map(|j| j.id.clone())
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            jobs.remove(&id);
        }

        Ok(count)
    }
}

/// Job worker for processing jobs
pub struct JobWorker<Q: JobQueue> {
    queue: Arc<Q>,
    queue_name: String,
    handlers: HashMap<String, Box<dyn JobHandler>>,
}

/// Handler for a specific job type
#[async_trait]
pub trait JobHandler: Send + Sync {
    async fn handle(&self, args: serde_json::Value) -> JobResult<()>;
}

impl<Q: JobQueue> JobWorker<Q> {
    pub fn new(queue: Arc<Q>, queue_name: impl Into<String>) -> Self {
        Self {
            queue,
            queue_name: queue_name.into(),
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a job type
    pub fn register<H: JobHandler + 'static>(&mut self, job_type: impl Into<String>, handler: H) {
        self.handlers.insert(job_type.into(), Box::new(handler));
    }

    /// Process one job (returns true if a job was processed)
    pub async fn process_one(&self) -> JobResult<bool> {
        let job = match self.queue.dequeue(&self.queue_name).await? {
            Some(job) => job,
            None => return Ok(false),
        };

        let handler = match self.handlers.get(&job.job_type) {
            Some(h) => h,
            None => {
                let mut failed_job = job;
                failed_job.mark_failed(format!("Unknown job type: {}", failed_job.job_type));
                self.queue.update(&failed_job).await?;
                return Ok(true);
            }
        };

        let mut job = job;
        match handler.handle(job.args.clone()).await {
            Ok(()) => {
                job.mark_completed();
            }
            Err(e) => {
                job.mark_failed(e.to_string());
            }
        }

        self.queue.update(&job).await?;
        Ok(true)
    }

    /// Run the worker loop
    pub async fn run(&self, shutdown: tokio::sync::watch::Receiver<bool>) {
        loop {
            if *shutdown.borrow() {
                break;
            }

            match self.process_one().await {
                Ok(true) => {
                    // Processed a job, continue immediately
                    continue;
                }
                Ok(false) => {
                    // No jobs, wait a bit
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    tracing::error!("Job worker error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_and_dequeue() {
        let queue = MemoryJobQueue::new();

        let job = Job::new("test_job", serde_json::json!({"key": "value"}));
        let job_id = queue.enqueue(job).await.unwrap();

        let dequeued = queue.dequeue("default").await.unwrap();
        assert!(dequeued.is_some());

        let job = dequeued.unwrap();
        assert_eq!(job.id, job_id);
        assert_eq!(job.status, JobStatus::Running);
    }

    #[tokio::test]
    async fn test_job_priority() {
        let queue = MemoryJobQueue::new();

        let low = Job::new("low", serde_json::json!({})).priority(JobPriority::Low);
        let high = Job::new("high", serde_json::json!({})).priority(JobPriority::High);

        queue.enqueue(low).await.unwrap();
        queue.enqueue(high).await.unwrap();

        let first = queue.dequeue("default").await.unwrap().unwrap();
        assert_eq!(first.job_type, "high");
    }

    #[tokio::test]
    async fn test_scheduled_jobs() {
        let queue = MemoryJobQueue::new();

        // Job scheduled for the future
        let future_job = Job::new("future", serde_json::json!({}))
            .run_in(3600); // 1 hour from now

        // Job ready now
        let now_job = Job::new("now", serde_json::json!({}));

        queue.enqueue(future_job).await.unwrap();
        queue.enqueue(now_job).await.unwrap();

        let dequeued = queue.dequeue("default").await.unwrap().unwrap();
        assert_eq!(dequeued.job_type, "now");

        // Future job should not be dequeued
        let next = queue.dequeue("default").await.unwrap();
        assert!(next.is_none());
    }

    #[tokio::test]
    async fn test_job_retry() {
        let mut job = Job::new("test", serde_json::json!({})).max_retries(3);

        job.mark_running();
        job.mark_failed("Error 1");

        assert_eq!(job.status, JobStatus::Retrying);
        assert_eq!(job.retries, 1);
        assert!(job.run_at.is_some());

        // Retry again
        job.status = JobStatus::Running;
        job.mark_failed("Error 2");
        assert_eq!(job.retries, 2);

        // Retry once more
        job.status = JobStatus::Running;
        job.mark_failed("Error 3");
        assert_eq!(job.retries, 3);

        // Should be dead now
        job.status = JobStatus::Running;
        job.mark_failed("Error 4");
        assert_eq!(job.status, JobStatus::Dead);
    }

    #[tokio::test]
    async fn test_pending_count() {
        let queue = MemoryJobQueue::new();

        queue.enqueue(Job::new("a", serde_json::json!({}))).await.unwrap();
        queue.enqueue(Job::new("b", serde_json::json!({}))).await.unwrap();
        queue.enqueue(Job::new("c", serde_json::json!({})).queue("other")).await.unwrap();

        let count = queue.pending_count("default").await.unwrap();
        assert_eq!(count, 2);
    }
}
