use redis::{aio::ConnectionManager, RedisError};

use crate::job::Job;
use anyhow::{Context, Result};

#[derive(Clone)]
pub struct RedisClient {
    connection: ConnectionManager,
    queue_name: String,
    unacked_queue_name: String,
}

impl RedisClient {
    pub async fn new() -> Result<RedisClient> {
        let client =
            redis::Client::open("redis://127.0.0.1/").context("failed to connect to redis")?;
        let manager = client
            .get_tokio_connection_manager()
            .await
            .context("failed to connect to redis manager")?;

        Ok(RedisClient {
            connection: manager,
            queue_name: "queue".to_owned(),
            unacked_queue_name: "unacked_queue".to_owned(),
        })
    }

    pub async fn rpop(&mut self, count: i32) -> Result<Vec<Job>> {
        let resp: Result<Vec<String>, RedisError> = redis::cmd("RPOP")
            .arg(&self.queue_name)
            .arg(count)
            .query_async(&mut self.connection)
            .await;

        match resp {
            Ok(job_strs) => {
                let result: Result<Vec<_>, _> = job_strs
                    .into_iter()
                    .map(|job_str| serde_json::from_str(&job_str).context("failed to decode job"))
                    .collect();

                result
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn lpush(&mut self, job: &Job) -> Result<()> {
        let job_str =
            serde_json::to_string(&job).with_context(|| format!("failed to serialize job json"))?;

        Ok(redis::cmd("LPUSH")
            .arg(&self.queue_name)
            .arg(&job_str)
            .query_async(&mut self.connection)
            .await?)
    }

    pub async fn rpoplpush(&mut self) -> Result<Option<Job>> {
        let resp: Result<Option<String>, RedisError> = redis::cmd("RPOPLPUSH")
            .arg(&self.queue_name)
            .arg(&self.unacked_queue_name)
            .query_async(&mut self.connection)
            .await;

        match resp {
            Ok(None) => Ok(None),
            Ok(Some(job_str)) => serde_json::from_str(&job_str).context("failed to decode job"),
            Err(err) => Err(err.into()),
        }
    }
}
