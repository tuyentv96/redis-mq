use std::sync::Arc;

use crate::{consumer::Consumer, job::Job, redis_client::RedisClient};
use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct Queue {
    redis_client: RedisClient,
    receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
    sender: mpsc::Sender<Job>,
    prefetch: i32,
}

impl Queue {
    pub fn new(redis_client: RedisClient, prefetch: i32) -> Queue {
        let (tx, rx) = mpsc::channel(prefetch as usize);
        Queue {
            redis_client,
            receiver: Arc::new(Mutex::new(rx)),
            sender: tx,
            prefetch: prefetch,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.batch_consume().await
    }

    pub fn new_consumer(&mut self) -> Consumer {
        Consumer::new(self.receiver.clone())
    }

    async fn batch_consume(&mut self) -> Result<()> {
        loop {
            let jobs = self.redis_client.rpop(self.prefetch).await?;

            if jobs.is_empty() {
                sleep(Duration::from_millis(1000)).await;
            } else {
                for job in jobs {
                    self.sender.send(job).await.context("failed to send job")?
                }
            }
        }
    }

    pub async fn publish(&mut self, job: &Job) -> Result<()> {
        self.redis_client.lpush(job).await
    }
}
