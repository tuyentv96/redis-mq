use std::{sync::Arc};

use crate::{
    job::Job,
};
use anyhow::Result;

use tokio::sync::Mutex;

use tokio::{sync::mpsc};

pub struct Consumer {
    receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
}

impl Consumer {
    pub fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Consumer {
        Consumer { receiver }
    }

    pub async fn fetch_task(&mut self) -> Result<Job> {
        Ok(self.receiver.lock().await.recv().await.unwrap())
    }
}
