use anyhow::Result;
use clap::Parser;
use indexmap::IndexMap;

mod args;
mod chapter_helper;
mod http_client;
mod save_helper;
mod service_invoker;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();

    let browser_api_url = format!("http://localhost:{}", args.port);

    println!("Starting Node Browser API");
    let mut browser_service =
        service_invoker::ServiceInvoker::new(args.port, args.silence_browser).await?;

    let chapters: Result<IndexMap<u32, String>> = async {
        println!("Establishing http client");
        let client = http_client::HTTPClient::new(&browser_api_url)?;

        println!("Fetching chapters");
        let links =
            chapter_helper::get_chapter_links(&client, &args.url, args.start, args.end).await?;

        println!("Found {} chapters", links.len());

        println!("Extracting {} chapters", links.len());

        let chapters = chapter_helper::extract_chapters(&client, &links).await?;

        println!("Successfully extracted {} chapters", chapters.len());

        return Ok(chapters);
    }
    .await;

    if let Err(err) = browser_service.shutdown().await {
        eprintln!("Failed to shutdown browser service: {}", err);
    }

    let chapters = chapters?;

    save_helper::remove_path(&args.out)?;

    if args.dir {
        save_helper::save_dir(chapters.clone(), &args.out)?;
    } else {
        save_helper::save_singlefile(chapters.clone(), &args.out)?;
    }

    println!("Saved chapters under '{}'", args.out);

    return Ok(());
}
