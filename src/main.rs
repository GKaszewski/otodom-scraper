use std::time::Duration;
use std::{env, error::Error};

use clap::Parser;
use clokwerk::{AsyncScheduler, Interval, TimeUnits};
use sqlx::{pool, PgPool};

mod scrapper;
mod web;

use scrapper::{clean_up_dead_offers, scrape_all_pages};
use web::get_web_app;

#[derive(Debug, Parser)]
#[clap(name = "Otodom Scraper", version = "1.0", author = "Gabriel Kaszewski")]
struct Config {
    #[clap(
        long,
        short = 't',
        default_value = "10",
        help = "Request rate in seconds"
    )]
    request_rate: Option<u64>,
    #[clap(
        long,
        short = 'i',
        default_value = "600",
        help = "Check interval in seconds"
    )]
    check_interval: Option<u32>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config: Config = Config::parse();

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let mut scheduler = AsyncScheduler::new();
    let interval = Interval::Seconds(config.check_interval.unwrap_or(600));
    let timeout = Duration::from_secs(config.request_rate.unwrap_or(10));
    //scrape_all_pages(timeout, pool).await?;

    println!(
        "Starting scraper with request rate: {}s and check interval: {:?}s",
        timeout.as_secs(),
        interval
    );

    let pool = pool.clone();
    let pool2 = pool.clone();
    scheduler.every(600.seconds()).run(move || {
        let pool = pool.clone();
        async move {
            match scrape_all_pages(timeout, pool.clone()).await {
                Ok(_) => println!("Scraping successful"),
                Err(e) => println!("Error: {:?}", e),
            }

            match clean_up_dead_offers(pool).await {
                Ok(_) => println!("Clean up successful"),
                Err(e) => println!("Error: {:?}", e),
            }
        }
    });

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // Start the web server
    let web_app = get_web_app(pool2);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:6420")
        .await
        .expect("Failed to bind to port 6420");
    axum::serve(listener, web_app).await.unwrap();
    println!("Server running on port 6420, press Ctrl+C to stop.");

    Ok(())
}
