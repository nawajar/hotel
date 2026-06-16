#!/usr/bin/env node
// Scans frontend/src for every t('key')/$t('key')/<i18n-t keypath="key"> usage
// and cross-checks it against the bundled locale catalogs.
//
//   node scripts/check-i18n-keys.mjs         # report only, exit 1 if any key
//                                             # used in code is missing from en.json
//   node scripts/check-i18n-keys.mjs --fix   # also auto-fill missing entries:
//                                             #   - en.json gets the raw key as a
//                                             #     placeholder value
//                                             #   - lo.json gets the (possibly just
//                                             #     fixed) English value as a placeholder
//
// Placeholders are meant to be visible and findable, not "done" - they exist so a
// forgotten key never silently renders blank or shows the raw dotted path in the UI.

import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SRC_DIR = path.join(__dirname, "..", "src");
const EN_PATH = path.join(SRC_DIR, "locales", "en.json");
const LO_PATH = path.join(SRC_DIR, "locales", "lo.json");
const FIX = process.argv.includes("--fix");

function walk(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const full = path.join(dir, entry);
    if (statSync(full).isDirectory()) {
      files.push(...walk(full));
    } else if (/\.(vue|ts)$/.test(entry)) {
      files.push(full);
    }
  }
  return files;
}

function flatten(obj, prefix = "", out = {}) {
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === "string") out[fullKey] = value;
    else flatten(value, fullKey, out);
  }
  return out;
}

function unflatten(flat) {
  const out = {};
  for (const [keyPath, value] of Object.entries(flat)) {
    const parts = keyPath.split(".");
    let node = out;
    for (let i = 0; i < parts.length - 1; i++) {
      node = node[parts[i]] ??= {};
    }
    node[parts[parts.length - 1]] = value;
  }
  return out;
}

function sortedFlat(flat) {
  return Object.fromEntries(Object.entries(flat).sort(([a], [b]) => a.localeCompare(b)));
}

// t('key'), t("key"), $t('key') - require a non-word char before "t(" so this
// doesn't match things like split("x") or other unrelated *t( calls.
const callPattern = /(?<![A-Za-z0-9_])\$?t\(\s*['"]([\w.]+)['"]/g;
// <i18n-t keypath="key">
const keypathPattern = /keypath=["']([\w.]+)["']/g;

const usedKeys = new Set();
for (const file of walk(SRC_DIR)) {
  if (file.startsWith(path.join(SRC_DIR, "locales"))) continue;
  const text = readFileSync(file, "utf8");
  for (const m of text.matchAll(callPattern)) usedKeys.add(m[1]);
  for (const m of text.matchAll(keypathPattern)) usedKeys.add(m[1]);
}

const en = flatten(JSON.parse(readFileSync(EN_PATH, "utf8")));
const lo = flatten(JSON.parse(readFileSync(LO_PATH, "utf8")));

const missingFromEn = [...usedKeys].filter((k) => !(k in en)).sort();
const missingFromLo = [...usedKeys].filter((k) => k in en && !(k in lo)).sort();
const unusedKeys = Object.keys(en)
  .filter((k) => !usedKeys.has(k))
  .sort();

if (missingFromEn.length) {
  console.log(`Used in code but missing from en.json (${missingFromEn.length}):`);
  for (const k of missingFromEn) console.log(`  - ${k}`);
}
if (missingFromLo.length) {
  console.log(`Used in code but missing from lo.json (${missingFromLo.length}, falls back to English):`);
  for (const k of missingFromLo) console.log(`  - ${k}`);
}
if (unusedKeys.length) {
  console.log(`In en.json but not referenced anywhere in src/ (${unusedKeys.length}):`);
  for (const k of unusedKeys) console.log(`  - ${k}`);
}
if (!missingFromEn.length && !missingFromLo.length && !unusedKeys.length) {
  console.log("All translation keys are in sync.");
}

if (FIX && (missingFromEn.length || missingFromLo.length)) {
  for (const k of missingFromEn) en[k] = k;
  for (const k of missingFromEn) lo[k] = en[k];
  for (const k of missingFromLo) lo[k] = en[k];

  writeFileSync(EN_PATH, JSON.stringify(unflatten(sortedFlat(en)), null, 2) + "\n");
  writeFileSync(LO_PATH, JSON.stringify(unflatten(sortedFlat(lo)), null, 2) + "\n");
  console.log("\n--fix applied: filled missing keys with placeholders. Review and translate them.");
  process.exit(0);
}

if (missingFromEn.length) {
  console.log("\nFailing: keys missing from en.json render as raw key text in the UI.");
  process.exit(1);
}

process.exit(0);
