"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { KIT } = require("./helpers");

const manifest = JSON.parse(fs.readFileSync(path.join(KIT, "hangar-kit.json"), "utf8"));
const read = (...parts) => fs.readFileSync(path.join(KIT, ...parts), "utf8");

function allFiles(dir = KIT, found = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if ([".git", ".hangar-ui", "node_modules", "target"].includes(entry.name)) continue; // target: native/'s build folder
    if (entry.isDirectory()) allFiles(path.join(dir, entry.name), found);
    else found.push(path.join(dir, entry.name));
  }
  return found;
}

test("the manifest's own fields", () => {
  assert.equal(manifest.schema, 2);
  assert.equal(manifest.id, "deps");
  assert.match(manifest.version, /^\d+\.\d+\.\d+$/);
  assert.ok(manifest.name.trim() && manifest.description.trim());
  assert.equal(manifest.homepage, "https://github.com/hangar-ai/kit-deps");
  assert.equal(manifest.hangar, ">=2.0.0");
  assert.ok(fs.existsSync(path.join(KIT, manifest.icon)));
  assert.equal(manifest.setup, undefined);
});

test("the gallery's card: one sentence, and its tags", () => {
  const { permissionsSummary, tags } = manifest.gallery;
  assert.ok(permissionsSummary.trim());
  assert.match(permissionsSummary, /command/);
  assert.ok(Array.isArray(tags) && tags.length > 0 && tags.every((tag) => typeof tag === "string" && tag.trim()));
});

test("the plugin is found at the root, named like the kit, with its skill", () => {
  assert.equal(manifest.plugin, undefined);
  const plugin = JSON.parse(read(".claude-plugin", "plugin.json"));
  assert.equal(plugin.name, manifest.id);
  assert.equal(plugin.version, manifest.version);
  assert.match(read("skills", "deps", "SKILL.md").replace(/\r\n/g, "\n"), /^---\nname: deps\ndescription: .+\n---\n/);
});

test("one declared command: node on the kit's own script, no placeholder but {kitDir}", () => {
  assert.deepEqual(Object.keys(manifest.commands), ["audit"]);
  const audit = manifest.commands.audit;
  assert.deepEqual(audit.run, ["node", "{kitDir}/tools/audit.js"]);
  assert.equal(audit.args, undefined);
  assert.ok(audit.description.trim());
  assert.ok(Number.isInteger(audit.timeoutSeconds) && audit.timeoutSeconds >= 1 && audit.timeoutSeconds <= 120);
  assert.ok(fs.existsSync(path.join(KIT, "tools", "audit.js")));
});

test("the tab: « Dépendances », and exactly the five permissions it needs", () => {
  assert.equal(manifest.tabs.length, 1);
  const [tab] = manifest.tabs;
  assert.match(tab.id, /^[a-z0-9-]{1,40}$/);
  assert.equal(tab.title, "Dépendances");
  assert.deepEqual([...tab.permissions].sort(), ["command:audit", "data.read", "data.write", "panes.read", "panes.send"]);
  assert.ok(fs.existsSync(path.join(KIT, tab.entry)));
  assert.ok(fs.existsSync(path.join(KIT, tab.icon)));
});

test("the tab is native: schema 2, the module as its entry, no page", () => {
  // An `entry` ending in .wasm needs schema 2 (docs/native/kits-native.md).
  assert.equal(manifest.schema, 2);
  const [tab] = manifest.tabs;
  assert.equal(tab.entry, "tabs/deps.wasm");
  assert.equal(tab.native, undefined);
  assert.ok(!path.isAbsolute(tab.entry) && !tab.entry.split(/[\\/]/).some((part) => part === ".." || part.startsWith(".")), "inside the kit, not hidden");
  const file = path.resolve(KIT, tab.entry);
  assert.ok(file.startsWith(KIT + path.sep));
  assert.ok(fs.existsSync(file), "the module is committed: Hangar never builds one");
  assert.ok(fs.statSync(file).size < 64 * 1024 * 1024);
  assert.equal(fs.existsSync(path.join(KIT, "tabs", "deps")), false, "no HTML page is left");
});

test("the module is WebAssembly for Hangar: its exports, and no WASI", () => {
  const bytes = fs.readFileSync(path.join(KIT, manifest.tabs[0].entry));
  assert.deepEqual([...bytes.subarray(0, 8)], [0, 0x61, 0x73, 0x6d, 1, 0, 0, 0]);
  const text = bytes.toString("latin1");
  for (const name of ["memory", "hangar_abi_version", "hangar_alloc", "hangar_init", "hangar_update", "hangar_view"]) assert.ok(text.includes(name), `exports ${name}`);
  assert.ok(text.includes("hangar") && !/wasi_snapshot_preview1|__wbindgen/.test(text), "built for wasm32-unknown-unknown: no WASI, no wasm-bindgen");
});

test("nothing personal anywhere in the kit", () => {
  const forbidden = /C:\\Users|\/Users\/|\/home\/\w|@[\w-]+\.(com|fr)|[Cc]o-[Aa]uthored/;
  for (const file of allFiles()) {
    if (file.includes(`${path.sep}fixtures${path.sep}`)) continue;
    assert.doesNotMatch(fs.readFileSync(file, "utf8"), forbidden, path.relative(KIT, file));
  }
});

test("the README says what it does and what it runs", () => {
  const readme = read("README.md");
  assert.match(readme, /## What it does/);
  assert.match(readme, /## What it runs/);
  for (const command of ["pnpm audit --json", "npm outdated --json", "cargo audit --json", "pip-audit", "pip list --outdated"]) assert.ok(readme.includes(command), command);
});
