use anyhow::{anyhow, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::Page;
use futures::StreamExt;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct Scraper {
    browser: Browser,
    handler_task: JoinHandle<()>,
}

impl Scraper {
    pub async fn new() -> Result<Self> {
        let config = BrowserConfig::builder()
            .no_sandbox()
            .args(vec![
                "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
             AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                "--disable-blink-features=AutomationControlled",
                "--disable-dev-shm-usage",
                "--disable-gpu",
                "--no-sandbox",
                "--disable-setuid-sandbox",
                "--disable-infobars",
                "--window-size=1920,1080",
            ])
            .build()
            .map_err(|e| anyhow!(e))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .context("Failed to launch browser")?;

        let handler_task =
            tokio::spawn(async move { while let Some(_event) = handler.next().await {} });

        return Ok(Self {
            browser,
            handler_task,
        });
    }

    pub async fn close(&mut self) -> Result<()> {
        self.browser.close().await?;
        self.handler_task.abort();
        return Ok(());
    }

    pub async fn get_page(&mut self, url: &str) -> Result<Page> {
        let page = self
            .browser
            .new_page("about:blank")
            .await
            .context("Failed to create page")?;

        page.evaluate(
            r#"
            Object.defineProperty(navigator, 'webdriver', { get: () => false });
        "#,
        )
        .await
        .context("Failed to apply stealth option")?;

        page.goto(url)
            .await
            .with_context(|| format!("Failed to navigate to {}", url))?;

        return Ok(page);
    }

    pub async fn get_links(page: &Page, filter: &str) -> Result<Vec<String>> {
        let js = format!(
            r#"
        (() => {{
            const regex = new RegExp({}, "i");

            return Array.from(document.querySelectorAll('a'))
                .map(a => a.href)
                .filter(href => regex.test(href));
        }})()
        "#,
            serde_json::to_string(filter)?
        );

        let links: Vec<String> = page
            .evaluate(js)
            .await
            .context("JS link evaluation failed")?
            .into_value()
            .unwrap_or_default();

        Ok(links)
    }
}

impl Drop for Scraper {
    fn drop(&mut self) {
        self.handler_task.abort();
    }
}
