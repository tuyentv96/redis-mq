
use std::time::Duration;

use anyhow::Result;
use futures::future::try_join_all;
use redis_mq::job::Job;
use redis_mq::queue::Queue;
use redis_mq::redis_client::RedisClient;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let redis = RedisClient::new().await?;
    let mut queue = Queue::new(redis, 10);

    let _t2 = tokio::spawn({
        let mut q = queue.clone();

        async move {
            let mut i: i32 = 0;
            loop {
                let job = Job { id: format!("{i}") };
                i += 1;

                q.publish(&job).await.unwrap();
                sleep(Duration::from_millis(5)).await;
            }
        }
    });

    let consumers: Vec<_> = (1..50)
        .into_iter()
        .map(|i| {
            let mut consumer = queue.new_consumer();

            tokio::spawn({
                async move {
                    loop {
                        let job = consumer.fetch_task().await;
                        println!("consumer_id: {}, consumed job {:?}", i, job)
                    }
                }
            })
        })
        .collect();

    queue.start().await?;
    try_join_all(consumers).await?;

    Ok(())
}
