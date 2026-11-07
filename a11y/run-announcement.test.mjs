// Exercise the actual cell UI with a deterministic kernel; no wasm or screen reader implied.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { JSDOM } from 'jsdom';

const script = readFileSync(new URL('../books/ikigai/js/run.js', import.meta.url), 'utf8');

for (const answer of ['long answer '.repeat(220), '<rss>\n<channel>News</channel>\n</rss>', '', 'one\n\nthree\n']) {
    test(`answer stays outside the live region (${answer.length} characters)`, { timeout: 5000 }, async () => {
        const dom = new JSDOM('<div class="ikigai-run" data-cmd="source urn:test"><pre class="ikigai-run-expected"></pre></div>', {
            url: 'https://book.invalid/', runScripts: 'outside-only',
        });
        try {
            const { window } = dom;
            if (window.document.readyState === "loading") {
                await new Promise(resolve => window.document.addEventListener("DOMContentLoaded", resolve, { once: true }));
            }
            window.testKernel = {
                default: async () => {},
                evalLineAsync: async () => JSON.stringify({ kind: 'output', text: answer }),
            };
            // Replace only the wasm import, leaving the UI and execution path intact.
            window.eval(script.replace('import(root + "wasm/book_wasm.js")', 'Promise.resolve(window.testKernel)'));
            const cell = window.document.querySelector('.ikigai-run');
            for (let run = 1; run <= 2; run++) {
                const finished = new Promise(resolve => window.document.addEventListener('ikigai:cell-ran', resolve, { once: true }));
                cell.querySelector('form').dispatchEvent(new window.Event('submit', { cancelable: true }));
                await finished;
                const pane = cell.querySelector('.ikigai-run-out');
                const live = cell.querySelector('[role="status"]');
                assert.ok(live, 'cell has a status region');
                assert.equal(pane.closest('[aria-live], [role="status"]'), null, 'full answer must not be announced');
                assert.equal(pane.textContent, answer && (answer.endsWith('\n') ? answer : answer + '\n'));
                assert.equal(pane.tabIndex, 0, 'answer is reachable by keyboard');
                assert.equal(pane.getAttribute('role'), 'region');
                assert.ok(pane.getAttribute('aria-label'), 'answer region has a name');
                const count = answer ? answer.replace(/\n$/, '').split('\n').length : 0;
                assert.match(live.textContent, new RegExp(`answered, ${count} line`));
                assert.match(live.textContent, new RegExp(`run ${run}`));
                assert.equal(window.document.activeElement, window.document.body, 'completion does not move focus');
            }
        } finally {
            dom.window.close();
        }
    });
}
