// Real WebAssembly traps, including a callback that leaves its exported promise pending.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { chromium } from 'playwright-core';
import { start } from './live-start.mjs';

const script = readFileSync(new URL('../books/ikigai/js/run.js', import.meta.url), 'utf8');
const stopped = "the page's kernel stopped after an earlier error; reload the page to run this";
const moduleSource = `
export default async function () {}
const trap = new WebAssembly.Instance(new WebAssembly.Module(new Uint8Array([
  0,97,115,109,1,0,0,0,1,4,1,96,0,0,3,2,1,0,
  7,8,1,4,116,114,97,112,0,0,10,5,1,3,0,0,11
]))).exports.trap;
export function evalLineAsync(line) {
  window.calls = (window.calls || 0) + 1;
  if (line === 'refusal') return Promise.resolve(JSON.stringify({kind:'error', text:'no endpoint resolved'}));
  if (line === 'broken') return Promise.reject(new Error('ordinary page error'));
  if (line === 'trap') {
    if (location.search.includes('callback')) {
      setTimeout(trap, 10);
      return new Promise(() => {});
    }
    if (location.search.includes('synchronous')) return trap();
    return Promise.resolve().then(trap);
  }
  if (location.search.includes('callback')) return new Promise(() => {});
  return Promise.resolve(JSON.stringify({kind:'output', text:'answer'}));
}
export const evalLineInChainAsync = (_, line) => evalLineAsync(line);
`;

for (const theme of ['light', 'rust', 'coal', 'navy', 'ayu']) {
    for (const mode of ['rejected', 'synchronous', 'callback', 'above', 'recoverable']) {
        test(`${theme}: ${mode === 'recoverable' ? 'ordinary errors keep the kernel usable' : mode + ' trap settles cells and fails the audit fast'}`, { timeout: 12000 }, async () => {
            const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chrome' });
            try {
                const page = await browser.newPage();
                await page.route('https://book.invalid/**', route => {
                    const path = new URL(route.request().url()).pathname;
                    const body = path.endsWith('run.js') ? script : path.endsWith('book_wasm.js') ? moduleSource
                        : `<html class="${theme}"><body>${['trap', 'later', 'last'].map(cmd =>
                            `<div class="ikigai-run" ${mode === 'above' ? 'data-game="test"' : ''} data-cmd="${cmd}"><pre class="ikigai-run-expected">expected ${cmd}</pre></div>`).join('')}<script src="/js/run.js"></script></body></html>`;
                    return route.fulfill({ contentType: path.endsWith('.js') ? 'text/javascript; charset=utf-8' : 'text/html; charset=utf-8', body });
                });
                await page.goto(`https://book.invalid/?${mode}`);
                await page.evaluate(() => window.ikigaiBookKernel());
                if (mode === 'recoverable') {
                    const cell = page.locator('.ikigai-run').first();
                    for (const command of ['refusal', 'broken', 'ok']) {
                        await cell.locator('textarea').fill(command);
                        await cell.getByRole('button', { name: 'Run', exact: true }).click();
                        await page.waitForFunction(() => document.querySelector('.ikigai-run-caption').textContent !== 'running…');
                        assert.equal(await cell.locator('.ikigai-run-button').isEnabled(), true);
                    }
                    assert.match(await cell.locator('.ikigai-run-out').textContent(), /answer/);
                    assert.equal(await page.evaluate(() => window.calls), 3);
                    return;
                }
                const began = Date.now();
                if (mode === 'above') {
                    await page.locator('.ikigai-run').nth(2).getByRole('button', { name: 'Run all above' }).click();
                } else {
                    // Same entry point and wait as the browser audit, with a short outer bound.
                    let timer;
                    const result = await Promise.race([start(page), new Promise(resolve => { timer = setTimeout(() => resolve('audit hung'), 2500); })])
                        .finally(() => clearTimeout(timer));
                    assert.equal(result, stopped, JSON.stringify(await page.locator(".ikigai-run-caption").allTextContents()));
                }
                await page.waitForFunction(sentence => [...document.querySelectorAll('.ikigai-run-caption')]
                    .every(c => c.textContent === sentence), stopped, { timeout: 1500 });
                assert.ok(Date.now() - began < 4000, 'does not wait for the 30-second audit timeout');
                assert.deepEqual(await page.locator('.ikigai-run-expected').allTextContents(), ['expected trap', 'expected later', 'expected last']);
                assert.equal(await page.locator('.ikigai-run-caption[role="status"][aria-live="polite"][aria-atomic="true"]').count(), 3);
                const calls = await page.evaluate(() => window.calls);
                // Enter and Reset still must not revive the dead instance or erase the reason.
                await page.locator('textarea').last().press('Enter');
                await page.locator('.ikigai-run').last().getByRole('button', { name: 'Reset', exact: true }).click();
                assert.equal(await page.evaluate(() => window.calls), calls);
                assert.deepEqual(await page.locator('.ikigai-run-caption').allTextContents(), [stopped, stopped, stopped]);
                assert.equal(await page.locator('button.ikigai-run-button:enabled').count(), 0);
            } finally {
                await browser.close();
            }
        });
    }
}
