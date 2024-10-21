use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use async_executor::{with_thread_pool, Executor};
use csv_scraper::{CsvScraper, ScrapeOperation};
use simple_error::SimpleResult;

struct SimpleScraper;

impl ScrapeOperation for SimpleScraper {
    fn execute(&self, _executor: Arc<Executor<'static>>) -> Pin<Box<dyn Future<Output = SimpleResult<String>> + Send + 'static>> {
        Box::pin(async { Ok("test".to_string()) })
    }
}

async fn async_main(ex: &Arc<Executor<'static>>) -> SimpleResult<()> {
    let csv_scraper = CsvScraper::new("test.csv", SimpleScraper).await?;
    csv_scraper.scrape(ex.clone(), Duration::from_secs(1)).await?;
    Ok(())
}

fn main() -> SimpleResult<()> {
    let ex = Arc::new(Executor::new());
    with_thread_pool(&ex, || async_io::block_on(async_main(&ex)))
}