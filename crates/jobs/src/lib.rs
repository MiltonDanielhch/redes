//! Ubicación: `crates/jobs/src/lib.rs
//!
//! Descripción: Módulo de jobs - procesamiento de tareas en background.
//!              Implementación con Tokio JoinSet + cola en memoria con back-pressure.
//!              Zero-dependency extra, solo usa tokio.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)
//!
//! # Arquitectura
//!
//! ```text
//! Request HTTP
//!      ↓
//! Enqueue Job → JobQueue (in-memory con back-pressure)
//!      ↓
//! Worker (tokio::task::spawn + JoinSet)
//!      ↓
//! Procesamiento async
//! ```

use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;

pub mod workers;

#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    pub max_concurrent: usize,
    pub max_buffer: usize,
}

impl Default for JobQueueConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            max_buffer: 1000,
        }
    }
}

pub struct JobQueue<T: Send + 'static> {
    sender: mpsc::Sender<Job<T>>,
    _phantom: std::marker::PhantomData<T>,
}

type BoxFuture = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>;

struct Job<T: Send + 'static> {
    payload: T,
    handler: Box<dyn FnOnce(T) -> BoxFuture + Send + 'static>,
}

impl<T: Send + 'static> JobQueue<T> {
    pub fn new(config: JobQueueConfig) -> Self {
        let (sender, receiver) = mpsc::channel::<Job<T>>(config.max_buffer);
        
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        
        tokio::spawn(Self::worker_loop(receiver, semaphore));
        
        Self { 
            sender, 
            _phantom: std::marker::PhantomData,
        }
    }

    async fn worker_loop(
        mut receiver: mpsc::Receiver<Job<T>>,
        semaphore: Arc<Semaphore>,
    ) {
        let mut join_set = JoinSet::new();
        
        loop {
            tokio::select! {
                Some(job) = receiver.recv() => {
                    let sem = semaphore.clone();
                    
                    join_set.spawn(async move {
                        let _permit = sem.acquire().await.expect("Semaphore closed");
                        (job.handler)(job.payload).await;
                    });
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(10)), if !join_set.is_empty() => {
                    if let Some(result) = join_set.join_next().await {
                        if let Err(e) = result {
                            tracing::error!("Job worker error: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    pub async fn enqueue<F, Fut>(&self, payload: T, handler: F) -> Result<(), QueueError>
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let job = Job {
            payload,
            handler: Box::new(move |p| Box::pin(handler(p))),
        };
        
        self.sender.send(job).await.map_err(|_| QueueError::QueueClosed)?;
        Ok(())
    }

    pub fn blocking_enqueue<F, Fut>(&self, payload: T, handler: F) -> Result<(), QueueError>
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let job = Job {
            payload,
            handler: Box::new(move |p| Box::pin(handler(p))),
        };
        
        self.sender.blocking_send(job).map_err(|_| QueueError::QueueClosed)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Queue is closed")]
    QueueClosed,
    #[error("Queue is full")]
    QueueFull,
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub struct JobBuilder<T: Send + 'static> {
    queue: JobQueue<T>,
}

impl<T: Send + 'static> JobBuilder<T> {
    pub fn new(config: JobQueueConfig) -> Self {
        Self {
            queue: JobQueue::new(config),
        }
    }

    pub async fn enqueue<F, Fut>(self, payload: T, handler: F) -> Result<(), QueueError>
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.queue.enqueue(payload, handler).await
    }
}

pub trait JobHandler<T: Send + 'static>: Send + Sync {
    fn handle(&self, payload: T) -> BoxFuture;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsAggregationJob {
    pub sede_id: i32,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlertDispatchJob {
    pub alert_id: i32,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupMetricsJob {
    pub older_than_days: i32,
}

impl EmailJob {
    pub fn new(to: impl Into<String>, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            subject: subject.into(),
            body: body.into(),
        }
    }
}

impl MetricsAggregationJob {
    pub fn new(sede_id: i32, period_start: String, period_end: String) -> Self {
        Self {
            sede_id,
            period_start,
            period_end,
        }
    }
}

impl AlertDispatchJob {
    pub fn new(alert_id: i32, channels: Vec<String>) -> Self {
        Self {
            alert_id,
            channels,
        }
    }
}

impl CleanupMetricsJob {
    pub fn new(older_than_days: i32) -> Self {
        Self { older_than_days }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_job_queue_basic() {
        let queue = JobQueue::<String>::new(JobQueueConfig {
            max_concurrent: 2,
            max_buffer: 100,
        });

        let results = Arc::new(std::sync::Mutex::new(Vec::new()));
        let results_clone = results.clone();

        queue.enqueue("job1".to_string(), move |payload| {
            let results = results_clone.clone();
            async move {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                results.lock().unwrap().push(payload);
            }
        }).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        assert_eq!(results.lock().unwrap().len(), 1);
        assert_eq!(results.lock().unwrap()[0], "job1");
    }

    #[tokio::test]
    async fn test_job_queue_concurrent() {
        let queue = JobQueue::<u32>::new(JobQueueConfig {
            max_concurrent: 5,
            max_buffer: 100,
        });

        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        
        for i in 0..10 {
            let counter = counter.clone();
            queue.enqueue(i, move |_payload| {
                async move {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }).await.unwrap();
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 10);
    }
}