import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, access } from 'node:fs/promises';
const index = JSON.parse(await readFile(new URL('../build/specs/index.json', import.meta.url)));
test('bundled catalog contains core tools and excludes Q-only specifications', async () => {
  for (const name of ['git','npm','docker','aws','cd']) {
    assert.ok(index.completions.includes(name), name);
    await access(new URL(`../build/specs/${name}.js`, import.meta.url));
  }
  for (const name of ['q','fig','amazon-q','kiro-cli']) assert.ok(!index.completions.includes(name));
});
test('all versioned definitions have a local index', async () => {
  for (const name of index.diffVersionedCompletions) {
    await access(new URL(`../build/specs/${name}/index.js`, import.meta.url));
  }
});
test('git command graph retains branch completion and has no AI generator', async () => {
  const {default: spec} = await import('../build/specs/git.js');
  const checkout = spec.subcommands.find(s => [s.name].flat().includes('checkout'));
  assert.ok([checkout.args].flat().some(arg => arg.generators));
  const commit = spec.subcommands.find(s => [s.name].flat().includes('commit'));
  const message = commit.options.find(o => [o.name].flat().includes('--message'));
  assert.deepEqual(message.args.generators, []);
});
test('catalog and application have no former Q CDN references', async () => {
  const loader = await readFile(new URL('../packages/autocomplete-parser/src/loadHelpers.ts', import.meta.url), 'utf8');
  const git = await readFile(new URL('../build/specs/git.js', import.meta.url), 'utf8');
  assert.doesNotMatch(loader, /amazonaws\.com|jupiter\.ai/);
  assert.doesNotMatch(git, /generate-recommendations|inline-shell-completion/);
});
