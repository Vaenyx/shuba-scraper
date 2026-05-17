use crate::scraper::Scraper;
use anyhow::{anyhow, Result};
use chromiumoxide::Page;
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

mod scraper;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long)]
    out: String,
}

fn get_book_id(url: &str) -> Result<String> {
    let regex = Regex::new(r"^https?://(?:www\.)?69shuba\.com/book/(\d+)(?:/)?$").unwrap();

    match regex.captures(url) {
        Some(captures) => Ok(captures.get(1).unwrap().as_str().to_string()),
        None => Err(anyhow!("Invalid 69shuba URL: {}", url)),
    }
}

async fn get_chapter_links(
    scraper: &mut Scraper,
    page: Page,
    url: &str,
    id: &str,
) -> Result<Vec<String>> {
    let page = scraper.get_content(page, url).await?;

    let regex = format!(r"^https:\/\/www\.69shuba\.com\/txt\/{}\/\d+$", id);

    let links = Scraper::get_links(&page, &regex).await;
    return links;
}

async fn get_chapters(
    scraper: &mut Scraper,
    links: Vec<String>,
) -> Result<HashMap<String, String>> {
    let chapters = HashMap::new();
    for link in links.iter() {
        let delay = rand::random_range(2000..5000);
        sleep(Duration::from_millis(delay)).await;

        let page = scraper.create_page().await?;

        let page = scraper.get_content(page, link).await?;

        let title = page
            .find_element(".txtnav h1")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();

        println!("{}", title);

        page.close().await.ok();
    }

    return Ok(chapters);
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let id = get_book_id(&args.url)?;

    let mut scraper = scraper::Scraper::new().await?;
    let page = scraper.create_page().await?;
    let links = get_chapter_links(&mut scraper, page.clone(), &args.url, &id).await?;

    println!("found {} links", links.len());
    get_chapters(&mut scraper, links.clone()).await?;

    sleep(Duration::from_millis(30000)).await;
    scraper.close().await?;

    std::fs::write(args.out, links.join("\n"))?;

    return Ok(());
}
