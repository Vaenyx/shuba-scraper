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
        let home = std::env::var("HOME")?;

        let user_data_dir = format!("{}/.config/google-chrome/shuba-scraper", home);

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
            .user_data_dir(user_data_dir)
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

    pub async fn create_page(&mut self) -> Result<Page> {
        let page = self
            .browser
            .new_page("about:blank")
            .await
            .context("Failed to create page")?;

        page.evaluate(
            r#"
(() => {
    //
    // webdriver
    //
    Object.defineProperty(navigator, 'webdriver', {
        get: () => undefined,
    });

    //
    // languages
    //
    Object.defineProperty(navigator, 'languages', {
        get: () => ['en-US', 'en'],
    });

    //
    // plugins
    //
    Object.defineProperty(navigator, 'plugins', {
        get: () => [
            {
                name: 'Chrome PDF Plugin',
                filename: 'internal-pdf-viewer',
                description: 'Portable Document Format'
            },
            {
                name: 'Chrome PDF Viewer',
                filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai',
                description: ''
            },
            {
                name: 'Native Client',
                filename: 'internal-nacl-plugin',
                description: ''
            }
        ],
    });

    //
    // chrome runtime
    //
    window.chrome = {
        runtime: {},
        app: {},
        csi: () => {},
        loadTimes: () => {},
    };

    //
    // permissions API
    //
    const originalQuery = window.navigator.permissions.query;

    window.navigator.permissions.query = (parameters) => (
        parameters.name === 'notifications'
            ? Promise.resolve({ state: Notification.permission })
            : originalQuery(parameters)
    );

    //
    // platform
    //
    Object.defineProperty(navigator, 'platform', {
        get: () => 'Win32',
    });

    //
    // hardwareConcurrency
    //
    Object.defineProperty(navigator, 'hardwareConcurrency', {
        get: () => 8,
    });

    //
    // deviceMemory
    //
    Object.defineProperty(navigator, 'deviceMemory', {
        get: () => 8,
    });

    //
    // maxTouchPoints
    //
    Object.defineProperty(navigator, 'maxTouchPoints', {
        get: () => 0,
    });

    //
    // vendor
    //
    Object.defineProperty(navigator, 'vendor', {
        get: () => 'Google Inc.',
    });

    //
    // userAgentData
    //
    Object.defineProperty(navigator, 'userAgentData', {
        get: () => ({
            brands: [
                { brand: 'Chromium', version: '136' },
                { brand: 'Google Chrome', version: '136' }
            ],
            mobile: false,
            platform: 'Windows'
        }),
    });

    //
    // WebGL spoofing
    //
    const getParameter = WebGLRenderingContext.prototype.getParameter;

    WebGLRenderingContext.prototype.getParameter = function(parameter) {
        //
        // UNMASKED_VENDOR_WEBGL
        //
        if (parameter === 37445) {
            return 'Intel Inc.';
        }

        //
        // UNMASKED_RENDERER_WEBGL
        //
        if (parameter === 37446) {
            return 'Intel Iris OpenGL Engine';
        }

        return getParameter.call(this, parameter);
    };

    //
    // outer dimensions
    //
    if (!window.outerWidth || !window.outerHeight) {
        window.outerWidth = window.innerWidth;
        window.outerHeight = window.innerHeight;
    }

    //
    // hairline fix
    //
    Object.defineProperty(screen, 'availTop', {
        get: () => 0,
    });

    //
    // fake mimeTypes
    //
    Object.defineProperty(navigator, 'mimeTypes', {
        get: () => [
            {
                type: 'application/pdf',
                suffixes: 'pdf',
                description: 'Portable Document Format'
            }
        ],
    });

    //
    // remove automation traces
    //
    delete window.__webdriver_script_fn;
    delete window.__driver_evaluate;
    delete window.__webdriver_evaluate;
    delete window.__selenium_evaluate;
    delete window.__fxdriver_evaluate;
    delete window.__driver_unwrapped;
    delete window.__webdriver_unwrapped;
    delete window.__selenium_unwrapped;
    delete window.__fxdriver_unwrapped;
    delete window._Selenium_IDE_Recorder;
    delete window._selenium;
    delete window.callSelenium;
    delete window.calledSelenium;
    delete window.domAutomation;
    delete window.domAutomationController;

    //
    // patch iframe detection
    //
    Object.defineProperty(HTMLIFrameElement.prototype, 'contentWindow', {
        get: function() {
            return window;
        }
    });

    //
    // fake media devices
    //
    if (navigator.mediaDevices) {
        navigator.mediaDevices.enumerateDevices = async () => {
            return [
                {
                    deviceId: 'default',
                    kind: 'audioinput',
                    label: 'Default Audio Device',
                    groupId: 'default'
                },
                {
                    deviceId: 'default',
                    kind: 'videoinput',
                    label: 'Default Video Device',
                    groupId: 'default'
                }
            ];
        };
    }

    console.log('Stealth patches applied');
})();
"#,
        )
        .await
        .context("Failed to apply stealth option")?;
        return Ok(page);
    }
    pub async fn get_content(&mut self, page: Page, url: &str) -> Result<Page> {
        page.goto(url)
            .await
            .with_context(|| format!("Failed to navigate to {}", url))?;

        // wait until body exists
        page.find_element("body").await?;

        // wait for cloudflare
        for _ in 0..30 {
            let title = page.get_title().await?.unwrap();

            if !title.contains("Just a moment") {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        Ok(page)
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

        return Ok(links);
    }
}

impl Drop for Scraper {
    fn drop(&mut self) {
        self.handler_task.abort();
    }
}
