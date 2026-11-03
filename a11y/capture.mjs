// Generated screenshots for a book: what any capture script needs, whatever page it shows (ledger
// #2973, and the path ledger #2023 reuses). `capture-dashboard.mjs` is the first to use it.
//
// The browser is the one axe-browser.mjs drives, for the same reason: `playwright-core` is pinned
// exactly and downloads nothing, the browser is the machine's installed Google Chrome or the binary
// `--chrome` names. A capture is committed, so the Chrome that made it is not part of any check.
//
// An image is WebP at a fixed quality, encoded by that same Chrome from its own screenshot (a canvas
// says `image/webp`), so regenerating needs no image tool beyond the browser that is already here.
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { chromium } from "playwright-core";

const QUALITY = 0.82;

// The browser, from the script's own arguments.
export async function launch(chromeAt) {
  const options = chromeAt ? { executablePath: chromeAt } : { channel: "chrome" };
  const browser = await chromium.launch(options);
  console.log(`browser: ${chromeAt ?? "Google Chrome (the installed one)"} ${browser.version()}`);
  return browser;
}

// A browser context signed in by opening a passkey invite, over Chrome's virtual authenticator: the
// real ceremony with no person and no hardware. The page has no theme yet; `url` below names one.
export async function signedIn(browser, invite, { width = 1280, height = 900 } = {}) {
  const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send("WebAuthn.enable");
  await cdp.send("WebAuthn.addVirtualAuthenticator", {
    options: {
      protocol: "ctap2",
      transport: "internal",
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
      automaticPresenceSimulation: true,
    },
  });
  await page.goto(invite);
  await page.fill("#enrol-label", "capture");
  await page.locator("#enrol-form button[type=submit]").click();
  await page.locator("#status .flash.ok").waitFor();
  await page.locator("#auth-login").click();
  await page.locator("#auth-who").waitFor();
  return { context, page };
}

// A directory of images, and a list of what went into it.
export class Shots {
  constructor(context, out) {
    this.out = out;
    this.written = [];
    this.encoder = context.newPage();
    mkdirSync(out, { recursive: true });
  }

  // `target` (a page's viewport, or a locator) as `name`.webp. A locator taller than `maxHeight` is
  // cut there, so a long table costs a look at its top and not a screen of rows no one can read.
  async take(target, name, { maxHeight } = {}) {
    const box = maxHeight ? await target.boundingBox() : null;
    const png =
      box && box.height > maxHeight
        ? await target.page().screenshot({
            type: "png",
            fullPage: true,
            clip: { x: box.x, y: box.y, width: box.width, height: maxHeight },
          })
        : await target.screenshot({ type: "png" });
    const encoder = await this.encoder;
    const url = `data:image/png;base64,${png.toString("base64")}`;
    const webp = await encoder.evaluate(
      async ({ url, quality }) => {
        const image = new Image();
        image.src = url;
        await image.decode();
        const canvas = document.createElement("canvas");
        canvas.width = image.naturalWidth;
        canvas.height = image.naturalHeight;
        canvas.getContext("2d").drawImage(image, 0, 0);
        return canvas.toDataURL("image/webp", quality).split(",")[1];
      },
      { url, quality: QUALITY },
    );
    const bytes = Buffer.from(webp, "base64");
    writeFileSync(join(this.out, `${name}.webp`), bytes);
    this.written.push({ name, bytes: bytes.length });
    console.log(`  ${name}.webp  ${(bytes.length / 1024).toFixed(0)} KiB`);
  }
}
