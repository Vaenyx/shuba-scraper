use anyhow::{anyhow, Result};
use clap::Parser;
use regex::Regex;

use crate::scraper::Scraper;
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

async fn get_chapter_links(scraper: &mut Scraper, url: &str, id: &str) -> Result<Vec<String>> {
    let page = scraper.get_page(url).await?;

    let regex = format!(r"^https:\/\/www\.69shuba\.com\/txt\/{}\/\d+$", id);

    return Scraper::get_links(&page, &regex).await;
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let id = get_book_id(&args.url)?;

    let mut scraper = scraper::Scraper::new().await?;
    let links = get_chapter_links(&mut scraper, &args.url, &id).await?;

    for link in links.iter() {
        println!("{}", link);
        scraper.get_page(&link).await?;
    }

    scraper.close().await?;

    std::fs::write(args.out, links.join("\n"))?;

    return Ok(());
}
