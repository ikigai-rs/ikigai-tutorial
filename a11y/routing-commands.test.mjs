// Run the chapter's actual config edits on GNU and BSD sed in CI.
import { readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync, readdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { test } from "node:test";

const root = fileURLToPath(new URL("../", import.meta.url));
const page = readFileSync(new URL("../books/gonk/src/ways/routing.md", import.meta.url), "utf8");
const commands = page.split("\n").filter(line => line.startsWith("$ cd lawnboy/gonk-worker && ") && line.includes("sed "));

test("routing config edits work with the platform's sed", () => {
    assert.equal(commands.length, 2);
    mkdirSync(root + "target", { recursive: true });
    const dir = mkdtempSync(root + "target/routing-commands-");
    const config = dir + "/lawnboy/gonk-worker/.gonk";
    try {
        mkdirSync(config, { recursive: true });
        writeFileSync(dir + "/hub.principal", "urn:iki:host:gonk:client:hub\n");
        for (const agent of ["claude", "codex"]) {
            writeFileSync(config + "/worker-" + agent + ".toml", 'agent = "' + agent + '"\nrouters = []\n');
        }
        execFileSync("sh", ["-c", commands[0].slice(2)], { cwd: dir });
        for (const agent of ["claude", "codex"]) {
            assert.equal(readFileSync(config + "/worker-" + agent + ".toml", "utf8"),
                'agent = "' + agent + '"\nrouters = ["urn:iki:host:gonk:client:hub"]\n\n[repos.library]\nremote = "https://github.com/acme/library.git"\n');
        }
        execFileSync("sh", ["-c", commands[1].slice(2)], { cwd: dir });
        assert.match(readFileSync(config + "/worker-claude.toml", "utf8"), /routers = \["urn:iki:host:gonk:client:hub", "urn:iki:gonk:passkey:QWRh"\]/);
        assert.equal(readdirSync(config).length, 2, "no backup files left behind");
    } finally {
        rmSync(dir, { recursive: true, force: true });
    }
});
