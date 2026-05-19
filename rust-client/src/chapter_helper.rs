use anyhow::{Result, anyhow};
use indexmap::IndexMap;
use rand::RngExt;
use regex::Regex;
use scraper::{Html, Selector};
use std::sync::LazyLock;
use tokio::time::{Duration, sleep};

use crate::http_client;

static BOOK_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://(?:www\.)?69shuba\.com/book/(\d+)/?$").unwrap());

fn get_book_id(url: &str) -> Result<String> {
    match BOOK_ID_REGEX.captures(url) {
        Some(captures) => Ok(captures.get(1).unwrap().as_str().to_string()),
        None => Err(anyhow!("Invalid 69shuba URL (how?): {}", url)),
    }
}

fn create_chapter_regex(book_id: &str) -> Result<Regex> {
    Ok(Regex::new(&format!(
        r"^https://www\.69shuba\.com/txt/{}/\d+$",
        regex::escape(book_id)
    ))?)
}

pub async fn get_chapter_links(
    client: &http_client::HTTPClient,
    url: &str,
    start: u32,
    end: u32,
) -> Result<IndexMap<u32, String>> {
    let response = client.fetch(url).await?;

    let parsed: http_client::FetchResponse = serde_json::from_str(&response)?;

    if !parsed.success {
        anyhow::bail!(
            "Browser API failed: {}",
            parsed.error.unwrap_or_else(|| "Unknown error".to_string())
        );
    }

    let html = parsed
        .html
        .ok_or_else(|| anyhow::anyhow!("Missing html field -> Internal Browser error"))?;

    let truncated_html = html
        .split(r#"<div class="contentadv">"#)
        .next()
        .unwrap_or(&html);

    let document = Html::parse_document(truncated_html);

    let selector = Selector::parse("a").map_err(|e| anyhow::anyhow!("{e}"))?;

    let book_id = get_book_id(url)?;
    let chapter_regex = create_chapter_regex(&book_id)?;

    let links: IndexMap<u32, String> = document
        .select(&selector)
        .filter_map(|el| {
            let href = el.value().attr("href")?;

            if chapter_regex.is_match(href) {
                Some(href.to_string())
            } else {
                None
            }
        })
        .enumerate()
        .map(|(i, val)| ((i as u32) + 1, val))
        .filter(|(idx, _)| *idx >= start && *idx <= end)
        .collect();

    return Ok(links);
}

async fn sleep_random(min: u64, max: u64) {
    let delay = rand::rng().random_range(min..=max);

    sleep(Duration::from_millis(delay)).await;

    return;
}

async fn extract_chapter(client: &http_client::HTTPClient, url: &str, idx: u32) -> Result<String> {
    let mut retries = 0;

    loop {
        println!("Extracting chapter {}", idx);

        let result: Result<String> = async {
            let response = client.fetch(url).await?;

            let parsed: http_client::FetchResponse = serde_json::from_str(&response)?;

            if !parsed.success {
                anyhow::bail!(
                    "Browser API failed: {}",
                    parsed.error.unwrap_or_else(|| "Unknown error".to_string())
                );
            }

            let html = parsed
                .html
                .ok_or_else(|| anyhow::anyhow!("Missing html field"))?;

            let document = Html::parse_document(&html);

            let title_selector =
                Selector::parse(".txtnav h1").map_err(|e| anyhow::anyhow!("{e}"))?;

            let content_selector =
                Selector::parse(".txtnav").map_err(|e| anyhow::anyhow!("{e}"))?;

            let title = document
                .select(&title_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();

            let content = document
                .select(&content_selector)
                .next()
                .map(|el| {
                    el.text()
                        .map(str::trim)
                        .filter(|text| {
                            !text.is_empty()
                                && !text.contains("loadAdv")
                                && !text.contains("window.")
                                && !text.contains("_taboola")
                                && !text.contains("tb_loader_script")
                                && !text.contains("performance.mark")
                                && !text.contains("flush: true")
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();

            let lines: Vec<_> = content.lines().collect();

            let cleaned_content = lines
                .iter()
                .skip(4)
                .take(lines.len().saturating_sub(6))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");

            return Ok(format!("{title}\n\n{cleaned_content}"));
        }
        .await;

        match result {
            Ok(chapter) => {
                return Ok(chapter);
            }

            Err(err) => {
                retries += 1;

                let cooldown = (30 * 2u64.pow(retries)).min(300) * 1000;

                eprintln!("Failed chapter {}: {}", idx, err);

                println!("Retrying in {} seconds...", cooldown / 1000);

                sleep(Duration::from_millis(cooldown)).await;
            }
        }
    }
}

pub async fn extract_chapters(
    client: &http_client::HTTPClient,
    links: &IndexMap<u32, String>,
) -> Result<IndexMap<u32, String>> {
    let mut chapters = IndexMap::new();

    for (idx, link) in links {
        if *idx % 50 == 0 {
            let cooldown = rand::random_range(100000..=250000);

            println!(
                "Cooling down for {} seconds (chapter {})",
                cooldown / 1000,
                idx
            );

            sleep(Duration::from_millis(cooldown)).await;
        }

        let chapter = extract_chapter(client, link, *idx).await?;

        chapters.insert(*idx, chapter);

        sleep_random(2500, 7000).await;
    }

    return Ok(chapters);
}
