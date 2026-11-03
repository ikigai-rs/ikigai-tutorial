// The images of the gonk Book's chapter "The Dashboard" (ledger #2973), from a real browser over a
// scratch gonk serving the committed seed, never a live gonk. `scripts/capture-dashboard.sh` runs
// this; see it for the arguments. `capture.mjs` is the part any other chapter's script reuses.
//
// Every image is a capture of the page gonk served. The order is the chapter's, and it matters: the
// stale entry needs a read and then a write, and the backfill scene leaves a ledger changed, so it
// comes last.
import { execFileSync, spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { launch, Shots, signedIn } from "./capture.mjs";

const args = process.argv.slice(2);
const named = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : null);
const [gonk, ikigai, seed, out] = ["--gonk", "--ikigai", "--seed", "--out"].map((flag) => {
  const value = named(flag);
  if (!value) {
    console.error("usage: node capture-dashboard.mjs --gonk <bin> --ikigai <bin> --seed <seed.py> --out <dir> [--chrome <path>]");
    process.exit(2);
  }
  return value;
});

const LEDGERS = ["app", "docs", "infra"];
const port = 20000 + Math.floor(Math.random() * 20000);
const base = `http://localhost:${port}`;
const scratch = mkdtempSync(join(tmpdir(), "gonk-dashboard-"));
const config = join(scratch, "config");
const homes = ["--config-home", config, "--data-home", join(scratch, "data")];
const socket = join(scratch, "data", "gonk.sock");
const importer = join(config, "gonk", "quic", "clients", "importer");
const sleep = (ms) => new Promise((done) => setTimeout(done, ms));
const run = (binary, words, input) =>
  execFileSync(binary, words, { input, encoding: "utf8", maxBuffer: 1 << 26, stdio: ["pipe", "pipe", "pipe"] });
// As the owner, over the socket: the reads and writes the chapter types.
const owner = (line) => run(ikigai, ["--mount", `urn:gk:=${socket}`, "-c", line]);

const grantLedgers = LEDGERS.flatMap((ledger) => ["--ledger", `${ledger}=write`]);
execFileSync("mkdir", ["-p", config]);
run(gonk, ["client", "add", "importer", ...grantLedgers, "--port", String(port), "--config-home", config]);
const invited = run(gonk, ["passkey", "invite", "operator", ...grantLedgers, "--port", String(port), ...homes]);
const invite = invited.match(/http:\/\/\S*#invite=\S+/)[0];
// The two tokens the operator's tabs ask for, as the chapter adds them.
const grantsFile = join(config, "gonk", "grants.json");
const grants = JSON.parse(readFileSync(grantsFile, "utf8"));
const holder = grants.grants ?? grants;
holder.operator = [...holder.operator, "urn:cap:kernel:inspect", "urn:cap:gonk:system:read"];
writeFileSync(grantsFile, JSON.stringify(grants, null, 2));

const server = spawn(
  gonk,
  [...homes, "--port", String(port), ...LEDGERS.flatMap((ledger) => ["--http-ledger", ledger])],
  { stdio: "ignore" },
);

const browser = await launch(named("--chrome"));
try {
  for (let tries = 0; ; tries++) {
    try {
      await (await fetch(`${base}/`)).body?.cancel();
      break;
    } catch (error) {
      if (tries >= 100) throw error;
      await sleep(100);
    }
  }
  run(gonk, ["ledger", "enroll", "importer", "--importer", ...homes]);
  const seeded = run(ikigai, ["--connect", `quic://127.0.0.1:${port}`, "--cert-dir", importer, "--quiet"], run("python3", [seed]));
  if (/^error/m.test(seeded)) throw new Error(`the seed was refused:\n${seeded}`);

  const { context, page } = await signedIn(browser, invite);
  const shots = new Shots(context, out);
  const open = async (path, theme) => {
    const url = new URL(path, base);
    url.searchParams.set("theme", theme);
    await page.goto(url.href);
    await page.waitForLoadState("networkidle");
  };
  const whole = page.locator("main");

  // Ledgers, in both schemes, then one chart, its table, and the proof the chart has no tooltip.
  for (const theme of ["light", "dark"]) {
    await open("/dashboard/ledgers", theme);
    await shots.take(whole, `ledgers-${theme}`);
  }
  await open("/dashboard/ledgers", "light");
  const chart = page.locator("figure.chart").first();
  await shots.take(chart, "chart-light");
  const box = await chart.locator(".chart-plot svg").first().boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  const tips = await chart.locator("svg title, [role=tooltip]").count();
  if (tips !== 0) throw new Error(`a chart has ${tips} tooltip(s) now: the chapter says it has none`);
  await page.mouse.move(0, 0);
  await chart.locator("details.chart-data summary").click();
  await shots.take(chart, "chart-table-light", { maxHeight: 640 });

  // Agents, a number's link, and the SPARQL page it opens.
  for (const theme of ["light", "dark"]) {
    await open("/dashboard/agents", theme);
    await shots.take(whole, `agents-${theme}`);
  }
  await open("/dashboard/agents", "light");
  const number = page.locator("td a").nth(1);
  await number.focus();
  await shots.take(page.locator(".panel").filter({ has: number }).first(), "agents-number-focused");
  const [opened] = await Promise.all([context.waitForEvent("page"), number.click()]);
  await opened.waitForLoadState("networkidle");
  await opened.setViewportSize({ width: 1280, height: 900 });
  await shots.take(opened, "agents-sparql-results");
  await opened.evaluate(() => window.scrollTo(0, 0));
  await shots.take(opened, "agents-sparql-query");
  await opened.close();

  // Polling, and its pause control.
  await open("/dashboard/agents", "light");
  await page.locator("#dashboard-pause").click();
  await shots.take(page.locator(".dashboard-live-control"), "paused");
  await page.locator("#dashboard-pause").click();

  // Cache: every read so far is cached, and a write cuts what hung from that ledger.
  await open("/dashboard/ledgers", "light");
  owner("sink urn:gk:iki:ledger:app:comment item=1 A note, so the cache has something stale.");
  await open("/dashboard/ledgers", "light");
  // A read the hub computes and does not keep, for "Reads not cached", and the System tab's backup.
  owner("source urn:gk:iki:gonk:backup");
  for (const theme of ["light", "dark"]) {
    await open("/dashboard/cache", theme);
    await shots.take(page, `cache-${theme}`);
  }
  await open("/dashboard/cache", "light");
  const part = (heading) => page.locator("section.readout").filter({ has: page.locator("h3", { hasText: heading }) });
  const stale = part("Cache entries").locator("tbody tr").filter({ has: page.locator("td", { hasText: /^cut$|^expired$/ }) });
  if ((await stale.count()) === 0) throw new Error("no stale entry to show");
  await stale.first().scrollIntoViewIfNeeded();
  await page.evaluate(() => window.scrollBy(0, 240));
  await shots.take(page, "cache-stale");
  console.log(`  stale entries: ${await stale.count()}; first: ${(await stale.first().innerText()).replace(/\s+/g, " ")}`);
  await shots.take(part("Reads not cached"), "cache-uncached", { maxHeight: 520 });
  console.log(`  not cached: ${(await part("Reads not cached").locator("tbody tr").first().innerText()).replace(/\s+/g, " ")}`);

  // System: the backup taken above gives its row something to say.
  for (const theme of ["light", "dark"]) {
    await open("/dashboard/system", theme);
    await shots.take(whole, `system-${theme}`);
  }

  // A ledger with a close it cannot date, then repaired. Last: this one changes `infra` for good.
  const graph = "urn:iki:ledger:graph:infra";
  const update =
    `PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#> WITH <${graph}> ` +
    "DELETE { ?item ledger:closedAt ?at } WHERE { ?item ledger:number 1 ; ledger:closedAt ?at }";
  owner(`sink urn:gk:iki:store:graph-update graph=${graph} ${update}`);
  await open("/dashboard/ledgers", "light");
  const refused = page.locator("figure.chart").filter({ hasText: "backfill-closed-at" });
  await shots.take(refused, "backfill-sentence-light");
  await open("/dashboard/ledgers", "dark");
  await shots.take(refused, "backfill-sentence-dark");
  owner("sink urn:gk:iki:ledger:infra:backfill-closed-at all");
  await open("/dashboard/ledgers", "light");
  await shots.take(page.locator("figure.chart").filter({ hasText: "infra" }), "backfill-repaired-light");

  console.log(`${shots.written.length} images in ${out}`);
} finally {
  await browser.close();
  server.kill();
  rmSync(scratch, { recursive: true, force: true });
}
