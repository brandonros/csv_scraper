use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_executor::Executor;
use async_io::Timer;
use simple_error::SimpleResult;

pub trait ScrapeOperation: Send + Sync {
    fn execute(&self, executor: Arc<Executor<'static>>) -> Pin<Box<dyn Future<Output = SimpleResult<()>> + Send + 'static>>;
}

pub struct Scraper;

impl Scraper {
    pub async fn scrape<T: ScrapeOperation + 'static>(
        executor: Arc<Executor<'static>>,
        delay: Duration,
        operation: T,
    ) -> SimpleResult<()> {
        loop {
            let op = operation.execute(executor.clone());
            let _handle = executor.spawn(async move {
                match op.await {
                    Ok(_) => (),
                    Err(err) => log::error!("error scraping: {err}")
                }
            });

            Timer::after(delay).await;
        }
    }
}
