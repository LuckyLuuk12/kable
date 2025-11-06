import fs from "fs";
import { inc, rcompare, parse } from "semver";

const explicitBumpType = process.argv[2];

// helpers
const readJson = (file) => {
  try { return JSON.parse(fs.readFileSync(file, "utf8")); }
  catch { return null; }
};

const readTomlVersion = (file) => {
  try {
    const content = fs.readFileSync(file, "utf8");
    const match = content.match(/version\s*=\s*"([^"]+)"/);
    return match ? match[1] : null;
  } catch { return null; }
};

const writeJson = (file, updater) => {
  const data = readJson(file);
  if (!data) return;
  updater(data);
  fs.writeFileSync(file, JSON.stringify(data, null, 2) + "\n");
};

const writeTomlVersion = (file, newVersion) => {
  try {
    let content = fs.readFileSync(file, "utf8");
    content = content.replace(/version\s*=\s*"[^"]+"/, `version = "${newVersion}"`);
    fs.writeFileSync(file, content);
  } catch {}
};

// read versions 
const pkg = readJson("package.json");
const tauri = readJson("src-tauri/tauri.conf.json");
const cargo = readTomlVersion("src-tauri/Cargo.toml");

// in Tauri v2, version is at root
const tauriVersion = tauri?.version;

const versions = [pkg?.version, tauriVersion, cargo].filter(Boolean);

if (versions.length === 0) {
  console.error("No versions found in package.json, tauri.conf.json, or Cargo.toml");
  process.exit(1);
}

// pick highest version in case of desync
versions.sort(rcompare);
const baseVersion = versions[0];

// determine bump type
let bumpType;
if (explicitBumpType) {
  // explicit bump type provided (e.g., npm run version:patch)
  bumpType = explicitBumpType;
} else {
  // auto-detect: if patch >= 9, bump minor instead
  const parsed = parse(baseVersion);
  if (parsed && parsed.patch >= 9) {
    bumpType = "minor";
  } else {
    bumpType = "patch";
  }
}

// bump
const newVersion = inc(baseVersion, bumpType);
console.log(`Bumping from ${baseVersion} → ${newVersion} (${bumpType})`);

// update files 
if (pkg) writeJson("package.json", (d) => { d.version = newVersion; });
if (tauri) writeJson("src-tauri/tauri.conf.json", (d) => { d.version = newVersion; });
writeTomlVersion("src-tauri/Cargo.toml", newVersion);

console.log("All versions updated successfully.");
