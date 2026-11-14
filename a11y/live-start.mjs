// Start the live content the browser accessibility audit checks.
const LIVE = "div.ikigai-run, div.ttt-play, div.ikigai-view, div.sheet-play, div.toolbox-check";

// Load the kernel and start everything live on the page. Returns a problem, or null.
export async function start(page) {
    if ((await page.locator(LIVE).count()) === 0) {
        return null;
    }
    const loaded = await page.evaluate(() =>
        window.ikigaiBookKernel().then(() => null, (e) => String(e && e.message ? e.message : e)));
    if (loaded !== null) {
        return `the in-page kernel did not load (${loaded}); build it first (scripts/build-kernels.sh)`;
    }
    // One snapshot of each set, clicked in the page: a toggle stops matching its selector the
    // moment it is pressed, so a re-evaluating locator would wait on it forever.
    await page.evaluate(() => {
        for (const run of document.querySelectorAll("button.ikigai-run-button:enabled")) {
            run.click();
        }
        for (const toggle of document.querySelectorAll('button.ikigai-view-toggle[aria-pressed="false"]')) {
            toggle.click();
        }
        for (const d of document.querySelectorAll("details.ikigai-run-expected-wrap")) {
            d.open = true;
        }
    });
    // Settled: no cell still running, and a polling view has had time to answer at least once.
    // A stages viewer (`js/stages.js`: the resilience harness, the live host) reads its picture
    // after its cells ran: wait for each one to say what the last one made, so axe sees the
    // stages and the alt text they came with.
    await page.waitForFunction(() =>
        [...document.querySelectorAll(".ikigai-run-caption")]
            .some((c) => c.textContent.includes("kernel stopped after an earlier error")) ||
        [...document.querySelectorAll(".ikigai-run-caption")].every((c) => c.textContent !== "running…") &&
        [...document.querySelectorAll(".ikigai-stages .ikigai-stages-status")]
            .every((s) => /No change|New stage|matches stage|could not be read/.test(s.textContent)),
        null, { timeout: 30000 });
    const stopped = await page.locator(".ikigai-run-caption").allTextContents();
    const failure = stopped.find((caption) => caption.includes("kernel stopped after an earlier error"));
    if (failure) {
        return failure;
    }
    await page.waitForTimeout(1200);
    // A "check this resource" (`js/toolbox.js`) is opened too: the RDF Toolbox, composed by the
    // page's kernel and written into a frame in place, with its engine loaded. axe then audits
    // what is in the frame along with the page (axe is put into every frame below). Ready means
    // the toolbox page says so (`data-ready` on its body) and the panel's status line agrees.
    const checks = await page.locator("button.toolbox-check-open").count();
    if ((await page.locator("div.toolbox-check").count()) !== checks) {
        return "a toolbox check did not offer its in-place panel although the kernel loaded";
    }
    if (checks > 0) {
        await page.evaluate(() => {
            for (const open of document.querySelectorAll('button.toolbox-check-open[aria-expanded="false"]')) {
                open.click();
            }
        });
        try {
            await page.waitForFunction(() =>
                [...document.querySelectorAll(".toolbox-panel")].every((panel) => {
                    const frame = panel.querySelector("iframe.toolbox-frame");
                    const body = frame && frame.contentDocument && frame.contentDocument.body;
                    return body && body.getAttribute("data-ready") === "true" &&
                        /ready/.test(panel.querySelector(".toolbox-panel-status").textContent);
                }), null, { timeout: 60000 });
        } catch (e) {
            const said = await page.evaluate(() =>
                [...document.querySelectorAll(".toolbox-panel-status")].map((s) => s.textContent).join(" | "));
            return `a toolbox panel never became ready: ${said}`;
        }
        await page.waitForTimeout(800);
    }
    return null;
}

