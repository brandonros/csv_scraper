use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_executor::{Executor, Task};
use async_fs::{File, OpenOptions};
use async_io::Timer;
use async_lock::Mutex;
use futures_lite::{AsyncWriteExt, StreamExt};
use simple_error::{box_err, SimpleResult};

pub trait ScrapeOperation: Send + Sync {
    fn execute(&self, executor: Arc<Executor<'static>>) -> Pin<Box<dyn Future<Output = SimpleResult<String>> + Send + 'static>>;
}

pub struct CsvScraper<T: ScrapeOperation> {
    operation: Arc<T>,
    file: Arc<Mutex<File>>,
}

impl<T: ScrapeOperation + 'static> CsvScraper<T> {
    pub async fn new(file_path: &str, operation: T) -> SimpleResult<Self> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .await?;
        Ok(Self {
            operation: Arc::new(operation),
            file: Arc::new(Mutex::new(file)),
        })
    }

    async fn append_to_file(file: &Arc<Mutex<File>>, data: &str) -> SimpleResult<()> {
        let mut file = file.lock().await;
        file.write_all(data.as_bytes()).await?;
        file.write_all("\n".as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    pub async fn scrape(
        self,
        executor: Arc<Executor<'static>>,
        delay: Duration,
    ) -> SimpleResult<()> {
        let mut timer = Timer::interval(delay);
        while let Ok(_) = timer.next().await.ok_or(box_err!("timer failed")) {
            let executor_clone = executor.clone();
            let operation = self.operation.clone();
            let file = self.file.clone();
            let handle: Task<SimpleResult<()>> = executor.spawn(async move {
                let csv_line = operation.execute(executor_clone).await?;
                Self::append_to_file(&file, &csv_line).await?;
                Ok(())
            });
            handle.detach();
        }
        Ok(())
    }
}