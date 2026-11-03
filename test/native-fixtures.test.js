"use strict";
// The native tab's tests (native/tests/recorded.rs) read what this script prints, from the recorded
// outputs of the package managers: the two halves of the kit are held to one document. Run with
// UPDATE_NATIVE_FIXTURES=1 to write the files again after the script's document changes.
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const audit = require("../tools/audit");
const { fixture, tempFolder, KIT } = require("./helpers");

const AT = Date.parse("2026-10-31T17:02:00Z");
const out = (stdout, code = 1) => ({ code, stdout, stderr: "", timedOut: false });

async function document(t, files, answers) {
  const dir = tempFolder(t);
  for (const [name, text] of Object.entries(files)) fs.writeFileSync(path.join(dir, name), text);
  const runner = async (program, args) => {
    const key = [program, ...args].join(" ");
    if (!(key in answers)) throw new Error(`unexpected command: ${key}`);
    return answers[key];
  };
  return audit.run(dir, { runner, now: () => AT });
}

const CASES = {
  "audit-npm.json": (t) =>
    document(t, { "package-lock.json": fixture("npm-package-lock.json") }, {
      "npm audit --json": out(fixture("npm-audit.json")),
      "npm outdated --json": out(fixture("npm-outdated.json")),
    }),
  "audit-multi.json": (t) =>
    document(t, { "pnpm-lock.yaml": "", "Cargo.lock": "", "requirements.txt": fixture("requirements.txt") }, {
      "pnpm audit --json": out(fixture("pnpm-audit.json")),
      "pnpm outdated --format json": out(fixture("pnpm-outdated.json"), 0),
      "cargo audit --json": out(fixture("cargo-audit.json")),
      "cargo outdated --format json --workspace --exit-code 0": out(fixture("cargo-outdated.json"), 0),
      "pip-audit -r requirements.txt --no-deps --disable-pip -f json": out(fixture("pip-audit.json")),
      "pip list --outdated --format json": out(fixture("pip-list-outdated.json"), 0),
    }),
};

for (const [name, make] of Object.entries(CASES)) {
  test(`native/tests/fixtures/${name} is what the audit script prints for the recorded outputs`, async (t) => {
    const file = path.join(KIT, "native", "tests", "fixtures", name);
    const printed = `${JSON.stringify(await make(t))}\n`;
    if (process.env.UPDATE_NATIVE_FIXTURES) fs.writeFileSync(file, printed);
    assert.equal(fs.readFileSync(file, "utf8"), printed, "run again with UPDATE_NATIVE_FIXTURES=1 and commit the result");
  });
}
