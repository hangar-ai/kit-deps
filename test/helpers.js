"use strict";
// Shared by the tests: the recorded outputs and a temp folder.
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const KIT = path.join(__dirname, "..");

/** A recorded output, by file name: see fixtures/. */
const fixture = (name) => fs.readFileSync(path.join(__dirname, "fixtures", name), "utf8");

function tempFolder(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "deps-kit-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

module.exports = { KIT, fixture, tempFolder };
