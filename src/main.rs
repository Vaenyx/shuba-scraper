use crate::scraper::Scraper;
use anyhow::{anyhow, Ok, Result};
use chromiumoxide::Page;
use clap::Parser;
use indexmap::IndexMap;
use regex::Regex;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};

mod scraper;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long)]
    out: String,

    #[arg(short, long)]
    start: Option<i32>,

    #[arg(short, long)]
    end: Option<i32>,

    #[arg(long)]
    dir: bool,
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
    chapter_map: IndexMap<usize, String>,
) -> Result<IndexMap<usize, String>> {
    let mut chapters: IndexMap<usize, String> = IndexMap::new();

    for (idx, link) in &chapter_map {
        loop {
            let delay = rand::random_range(3500..7500);
            sleep(Duration::from_millis(delay)).await;

            println!("Extracting chapter {}", idx);

            let result: Result<String> = async {
                let page = scraper.create_page().await?;
                let page = scraper.get_content(page, link).await?;

                let title = page
                    .find_element(".txtnav h1")
                    .await?
                    .inner_text()
                    .await?
                    .unwrap_or_default();

                let content = page
                    .find_element(".txtnav")
                    .await?
                    .inner_text()
                    .await?
                    .unwrap_or_default();

                let combined = format!("{}\n\n{}", title, content);

                page.close().await.ok();

                Ok(combined)
            }
            .await;

            match result {
                std::result::Result::Ok(chapter) => {
                    chapters.insert(*idx, chapter);
                    break;
                }
                Err(err) => {
                    eprintln!(
                        "Failed extracting chapter {}: {}. Retrying in 60 seconds...",
                        idx, err
                    );
                    sleep(Duration::from_secs(60)).await;
                }
            }
        }
    }

    Ok(chapters)
}

fn remove_path(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);

    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)?;

    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    return Ok(());
}

fn save_singlefile(chapters: IndexMap<usize, String>, out: &str) -> Result<()> {
    let text = chapters
        .values()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n\n\n");

    std::fs::write(out, text)?;
    return Ok(());
}

fn save_dir(chapters: IndexMap<usize, String>, out: &str) -> Result<()> {
    std::fs::create_dir_all(out)?;
    for (idx, text) in chapters {
        std::fs::write(format!("{}/{}", out, idx), text)?;
    }
    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let start = args.start.unwrap_or(1);
    let end = args.end.unwrap_or(i32::MAX);
    let id = get_book_id(&args.url)?;

    let mut scraper = scraper::Scraper::new().await?;
    let page = scraper.create_page().await?;
    let links = get_chapter_links(&mut scraper, page.clone(), &args.url, &id).await?;

    let chapter_map: IndexMap<usize, String> = links
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, val)| (i + 1, val))
        .filter(|(idx, _)| *idx >= start as usize && *idx <= end as usize)
        .collect();

    println!("found {} chapters", chapter_map.len());

    let chapters = get_chapters(&mut scraper, chapter_map.clone()).await?;

    scraper.close().await?;

    remove_path(&args.out)?;
    if args.dir {
        save_dir(chapters.clone(), &args.out)?;
    } else {
        save_singlefile(chapters.clone(), &args.out)?;
    }

    println!("Extraction complete");
    return Ok(());
}
