const express = require("express");
const cheerio = require("cheerio");

const { chromium } = require("playwright-extra");

const StealthPlugin = require("puppeteer-extra-plugin-stealth");

chromium.use(StealthPlugin());

const app = express();

app.use(express.json());

let browserContext;

async function startBrowser() {
  browserContext =
    await chromium.launchPersistentContext(
      "../chrome-profile",
      {
        headless: true,

        viewport: {
          width: 1920,
          height: 1080,
        },

        locale: "en-US",

        userAgent:
          "Mozilla/5.0 (Windows NT 10.0; Win64; x64) " +
          "AppleWebKit/537.36 (KHTML, like Gecko) " +
          "Chrome/136.0.0.0 Safari/537.36",

        args: [
          "--start-maximized",
          "--disable-dev-shm-usage",
          "--no-sandbox",
          "--disable-gpu",
        ],
      }
    );

  console.log("Browser started");
}

function extractFirstTxtnav(html) {
  const $ = cheerio.load(html);

  const el = $(".txtnav").first();

  if (!el.length) {
    return null;
  }

  let result = "";

  for (const node of el[0].children) {
    // skip ads
    if (
      node.type === "tag" &&
      (
        node.attribs?.class?.includes("bottom-ad") ||
        node.attribs?.class?.includes("contentadv")
      )
    ) {
      continue;
    }

    if (node.type === "text") {
      result += node.data;
    }

    if (node.name === "br") {
      result += "\n";
    }

    // include text from normal tags
    if (
      node.type === "tag" &&
      node.name !== "script"
    ) {
      result += $(node).text();
    }
  }

  return result.trim();
}

function randomInt(min, max) {
  return Math.floor(
    Math.random() * (max - min + 1)
  ) + min;
}

async function sleep(ms) {
  return new Promise(resolve =>
    setTimeout(resolve, ms)
  );
}

async function humanize(page) {
  try {
    await page.mouse.move(
      randomInt(100, 800),
      randomInt(100, 800),
      { steps: randomInt(10, 30) }
    );

    await sleep(randomInt(500, 1500));

    await page.mouse.wheel(0, randomInt(200, 1000));

    await sleep(randomInt(1000, 3000));

    if (Math.random() > 0.5) {
      await page.keyboard.press("PageDown");
    }
  } catch { }
}

async function waitForCloudflare(page) {
  while (true) {
    const title = await page.title().catch(() => "");
    const html = await page.content().catch(() => "");

    const blocked =
      title.includes("Just a moment") ||
      html.includes("challenge-platform") ||
      html.includes("cf-browser-verification") ||
      html.includes("turnstile") ||
      html.includes("Cloudflare");

    if (!blocked) {
      break;
    }

    console.log("Cloudflare challenge detected. Waiting...");

    await humanize(page);

    await sleep(randomInt(5000, 10000));
  }
}

app.post("/fetch", async (req, res) => {
  const { url, txtnav } = req.body;

  if (!url) {
    return res.status(400).json({
      success: false,
      error: "Missing url",
    });
  }

  let page;

  try {
    page = await browserContext.newPage();

    await humanize(page);

    console.log(`Navigating to ${url}`);

    await page.goto(url, {
      waitUntil: "domcontentloaded",
      timeout: 120000,
    });

    await waitForCloudflare(page);

    await page.waitForLoadState("networkidle");

    await sleep(3000);

    const html = await page.content();
    const title = await page.title();

    let txtnav_text = null;

    if (txtnav) {
      txtnav_text = extractFirstTxtnav(html);
    }

    await humanize(page);

    return res.json({
      success: true,
      title,
      html,
      ...(txtnav && { txtnav_text }),
    });
  } catch (err) {
    return res.status(500).json({
      success: false,
      title: null,
      error: err.message,
    });
  } finally {
    if (page) {
      await page.close().catch(() => { });
    }
  }
});

(async () => {
  await startBrowser();

  const port = process.argv[2] || 3000;

  app.listen(port, () => {
    console.log(`Browser API listening on port ${port}`);
    console.log("READY");
  });
})();

process.stdout.on("error", err => {
  if (err.code === "EPIPE") {
    return;
  }

  throw err;
});
