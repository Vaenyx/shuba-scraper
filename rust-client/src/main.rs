use anyhow::{Ok, Result};

mod http_client;
mod service_invoker;

#[tokio::main]
async fn main() -> Result<()> {
    let port: usize = 3004;
    let browser_api_url = format!("http://localhost:{}", port);

    println!("Starting Node Browser API");
    let mut browser_service = service_invoker::ServiceInvoker::new(port).await?;

    println!("Establishing http client");
    let client = http_client::HTTPClient::new(&browser_api_url)?;

    println!("Calling api");

    //let response = client.fetch("https://www.69shuba.com/book/32979").await?;
    let response = client.fetch("https://www.google.de").await?;

    println!("{}", response);

    browser_service.shutdown().await?;

    return Ok(());
}
