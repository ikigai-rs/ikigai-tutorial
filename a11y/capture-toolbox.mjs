// The images of The ikigai Book's chapter "Checking a resource" (ledger #2980), from a real browser
// over scratch hosts serving the chapter's own crate and a toolbox build (`check-resource`'s example
// `toolbox_host`), never a live gonk. `scripts/capture-toolbox.sh` runs this; see it for the
// arguments. `capture.mjs` is the part it shares with the Dashboard's script.
//
// Every image is a capture of the page the host served, and each scene first checks that the page
// says what the chapter says it says, so a toolbox that words a finding differently stops here
// rather than in a picture that no longer matches its alt text.
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { launch, Shots } from "./capture.mjs";

const args = process.argv.slice(2);
const named = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : null);
const [hostBinary, dist, out] = ["--host", "--dist", "--out"].map((flag) => {
  const value = named(flag);
  if (!value) {
    console.error("usage: node capture-toolbox.mjs --host <toolbox_host> --dist <build> --out <dir> [--chrome <path>]");
    process.exit(2);
  }
  return value;
});

const PLAYER_O = "urn:iki:tutorial:check:player:o";
const PLAYER_X = "urn:iki:tutorial:check:player:x";
const SHAPES = "urn:iki:tutorial:check:shapes";
const REFUSAL = `denied: capability does not grant \`urn:cap:tutorial:players:read\` (declared by \`${PLAYER_O}\`)`;
// A question about the same resource, on one line so that Format has a layout to change.
const QUERY = "PREFIX ttt: <http://example.org/ttt#>\nSELECT ?player ?wins WHERE { ?player a ttt:Player ; ttt:wins ?wins }\n";
const THEMES = ["light", "dark"];

// One host and its one caller. It prints its address when it is listening.
const hosts = [];
const host = (...scopes) =>
  new Promise((listening, failed) => {
    const child = spawn(hostBinary, [dist, ...scopes], { stdio: ["ignore", "pipe", "inherit"] });
    hosts.push(child);
    child.once("error", failed);
    child.once("exit", (code) => failed(new Error(`the host exited with ${code} before it listened`)));
    createInterface({ input: child.stdout }).once("line", listening);
  });

const says = async (locator, words) => {
  const text = (await locator.innerText()).replace(/\s+/g, " ");
  if (!text.includes(words)) throw new Error(`expected "${words}" in: ${text}`);
};

const browser = await launch(named("--chrome"));
try {
  const reader = await host("urn:cap:tutorial:players:read");
  const stranger = await host("urn:cap:tutorial:counter:write");

  const context = await browser.newContext({ viewport: { width: 1040, height: 560 }, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const shots = new Shots(context, out);
  // The check page for `iri`, `shapes=` when it has some, with the engine's first answer in.
  const open = async (base, iri, theme, { shapes, tool } = {}) => {
    const query = new URLSearchParams({ theme });
    if (shapes) query.set("shapes", shapes);
    await page.goto(`${base}/toolbox/check/${encodeURIComponent(iri)}?${query}${tool ? `#${tool}` : ""}`);
    await page.locator("body[data-ready=true][data-busy=false]").waitFor();
  };
  const tab = async (tool) => {
    await page.locator(`[data-tool=${tool}]`).click();
    await page.locator(`#tool[data-tool=${tool}]`).waitFor();
  };
  const result = page.locator("#tool");

  for (const theme of THEMES) {
    // One resource, no shapes: Validate, then Format and SPARQL over the same input.
    await open(reader, PLAYER_O, theme);
    await says(page.locator("#files"), PLAYER_O);
    await says(result, "0 problems");
    await says(page.locator("#note:not([hidden])"), "The SHACL tab appears when the input has shapes.");
    await shots.take(page, `validate-${theme}`);

    await tab("format");
    await says(result, "Same graph");
    await shots.take(page, `format-${theme}`);

    await tab("sparql");
    await says(result, "No queries in the input.");
    await page.locator("#paste").click();
    await page.locator("#paste-text").fill(QUERY);
    await page.locator("#paste-add").click();
    await page.locator("body[data-busy=false] #tool").filter({ hasText: "Same SPARQL algebra" }).waitFor();
    await says(result, "0 problems");
    await shots.take(page, `sparql-${theme}`);

    // Against the shapes: O's one finding, and X with none.
    await open(reader, PLAYER_O, theme, { shapes: SHAPES, tool: "shacl" });
    await says(result, "1 violation");
    await says(result, `ttt:wins is not allowed on <${PLAYER_O}> by the closed shape ttt:PlayerShape`);
    await shots.take(page, `shacl-o-${theme}`);

    await open(reader, PLAYER_X, theme, { shapes: SHAPES, tool: "shacl" });
    await says(result, "The data conforms to the shapes.");
    await shots.take(page.locator(".pane.tool"), `shacl-x-${theme}`);

    // A caller without the grant, while the page's note is up: it lasts six seconds.
    await page.goto(`${stranger}/toolbox/check/${encodeURIComponent(PLAYER_O)}?shapes=${encodeURIComponent(SHAPES)}&theme=${theme}#shacl`);
    await page.locator("body[data-ready=true][data-busy=false]").waitFor();
    await says(page.locator("#crash:not([hidden])"), "HTTP 403");
    await says(page.locator("#files"), SHAPES);
    await says(result, "The data conforms to the shapes.");
    await shots.take(page, `refused-${theme}`);
    // And after it, the words the chapter warns about, with no data in the input at all.
    await page.locator("#crash[hidden]").waitFor({ state: "attached" });
    await says(result, "The data conforms to the shapes.");
  }

  // The refusal's own words, at the address the page asked for. The browser draws this one.
  await page.setViewportSize({ width: 1040, height: 70 });
  const refused = await page.goto(`${stranger}/toolbox/input/${encodeURIComponent(PLAYER_O)}`);
  if (refused.status() !== 403) throw new Error(`the input answered ${refused.status()}, not 403`);
  await says(page.locator("body"), REFUSAL);
  await shots.take(page, "refused-input");

  console.log(`${shots.written.length} images in ${out}`);
} finally {
  await browser.close();
  for (const child of hosts) {
    child.removeAllListeners("exit");
    child.kill();
  }
}
