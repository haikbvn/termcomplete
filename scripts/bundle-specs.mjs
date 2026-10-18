import { build } from 'esbuild';
import ts from 'typescript';
import { readdir, readFile, mkdir, rm, copyFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
process.chdir(fileURLToPath(new URL('..', import.meta.url)));

const root = path.resolve('vendor/specs');
const output = path.resolve('build/specs');
const excluded = new Set(['q', 'fig', 'amazon-q', 'kiro-cli']);
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
const entries = {};
const completions = [];
const versioned = [];
async function walk(directory, prefix = '') {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const name = path.posix.join(prefix, entry.name);
    if (entry.isDirectory()) await walk(path.join(directory, entry.name), name);
    else if (entry.name.endsWith('.ts')) {
      const key = name.slice(0, -3);
      if (!excluded.has(key.split('/')[0])) entries[key] = path.join(directory, entry.name);
    }
  }
}
await walk(path.join(root, 'src'));
for (const entry of await readdir(path.join(root, 'src'), { withFileTypes: true })) {
  const name = entry.name.replace(/\.ts$/, '');
  if (excluded.has(name)) continue;
  if (entry.isFile() && entry.name.endsWith('.ts')) completions.push(name);
  else if (entry.isDirectory() && entries[`${name}/index`]) {
    completions.push(name); versioned.push(name);
  }
}
let removedAI = 0;
await build({
  entryPoints: entries, outdir: output, bundle: true, format: 'esm',
  platform: 'browser', target: 'safari15', treeShaking: true, minify: true,
  plugins: [{name: 'omit-ai', setup(builder) {
    builder.onLoad({filter: /\.ts$/}, async ({path: filename}) => {
      if (!filename.startsWith(path.join(root, 'src'))) return;
      let source = await readFile(filename, 'utf8');
      const ast = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true);
      const edits = [];
      function visit(node) {
        if (ts.isPropertyAssignment(node) && node.name.getText(ast) === 'generators' && ts.isCallExpression(node.initializer) && node.initializer.expression.getText(ast) === 'ai') {
          edits.push([node.getStart(ast), node.end]); removedAI++;
        }
        ts.forEachChild(node, visit);
      }
      visit(ast);
      for (const [start, end] of edits.sort((a,b) => b[0]-a[0])) source = source.slice(0,start) + 'generators: []' + source.slice(end);
      return {contents: source, loader: 'ts', resolveDir: path.dirname(filename)};
    });
  }}],
});
await writeFile(path.join(output, 'index.json'), JSON.stringify({completions: completions.sort(), diffVersionedCompletions: versioned.sort()}));
await copyFile(path.join(root, 'LICENSE'), path.join(output, 'LICENSE'));
await copyFile(path.join(root, 'UPSTREAM.json'), path.join(output, 'UPSTREAM.json'));
console.log(`Bundled ${completions.length} commands (${versioned.length} versioned); omitted ${removedAI} AI generators.`);

await mkdir(path.join(output, 'icons'), {recursive:true});
for (const dir of [path.join(root,'icons'), path.resolve('crates/fig_desktop/icons/autocomplete')]) {
  for (const name of await readdir(dir)) await copyFile(path.join(dir,name),path.join(output,'icons',name));
}

// Carry the licensed application builtin themes into the CLI's theme directory.
const themesSource = await readFile('packages/autocomplete-app/src/fig/themes.ts', 'utf8');
const themesAst = ts.createSourceFile('themes.ts', themesSource, ts.ScriptTarget.Latest, true);
let themes;
function findThemes(node) {
  if (ts.isVariableDeclaration(node) && node.name.getText(themesAst) === 'builtInThemes') themes = node.initializer;
  ts.forEachChild(node, findThemes);
}
findThemes(themesAst);
if (!themes || !ts.isObjectLiteralExpression(themes)) throw new Error('Builtin themes must be a literal object');
const {runInNewContext} = await import('node:vm');
const builtins = runInNewContext(`(${themes.getText(themesAst)})`, Object.create(null), {timeout:1000});
await mkdir('build/themes', {recursive:true});
for (const [name,value] of Object.entries(builtins)) await writeFile(`build/themes/${name}.json`, JSON.stringify(value));
await writeFile('build/themes/system.json', JSON.stringify({version:'1.0', name:'System'}));
