#!/usr/bin/env bun
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');

// 1. Determine target version
const rootPkgPath = path.join(rootDir, 'package.json');
const rootPkg = JSON.parse(fs.readFileSync(rootPkgPath, 'utf-8'));

let targetVersion = process.argv[2];
if (targetVersion) {
  // Strip optional 'v' prefix, e.g. v1.0 -> 1.0.0 or 1.0
  if (targetVersion.startsWith('v') || targetVersion.startsWith('V')) {
    targetVersion = targetVersion.slice(1);
  }
  // Ensure semver format: e.g. "1.0" -> "1.0.0"
  const parts = targetVersion.split('.');
  while (parts.length < 3) {
    parts.push('0');
  }
  targetVersion = parts.join('.');
  rootPkg.version = targetVersion;
  fs.writeFileSync(rootPkgPath, JSON.stringify(rootPkg, null, 2) + '\n');
  console.log(`[Swifter] Set root package.json version to ${targetVersion}`);
} else {
  targetVersion = rootPkg.version;
}

// 2. Sync root VERSION file
const versionFilePath = path.join(rootDir, 'VERSION');
fs.writeFileSync(versionFilePath, `${targetVersion}\n`);

// 3. Sync root Cargo.toml [workspace.package] version
const cargoTomlPath = path.join(rootDir, 'Cargo.toml');
if (fs.existsSync(cargoTomlPath)) {
  let cargoContent = fs.readFileSync(cargoTomlPath, 'utf-8');
  if (cargoContent.includes('[workspace.package]')) {
    cargoContent = cargoContent.replace(
      /(\[workspace\.package\][^]*?version\s*=\s*)"[^"]+"/,
      `$1"${targetVersion}"`
    );
  } else {
    // Inject [workspace.package] if not present
    const insertAfter = 'resolver = "2"\n';
    const workspacePkgBlock = `\n[workspace.package]\nversion = "${targetVersion}"\nauthors = ["msandypr"]\nedition = "2024"\nlicense = "GPL-3.0-only"\n`;
    if (cargoContent.includes(insertAfter)) {
      cargoContent = cargoContent.replace(insertAfter, insertAfter + workspacePkgBlock);
    } else {
      cargoContent = cargoContent + workspacePkgBlock;
    }
  }
  fs.writeFileSync(cargoTomlPath, cargoContent);
}

// 4. Sync subpackage package.json files
const subPackages = ['apps/desktop/package.json', 'apps/ui/package.json'];
for (const relPath of subPackages) {
  const fullPath = path.join(rootDir, relPath);
  if (fs.existsSync(fullPath)) {
    const pkg = JSON.parse(fs.readFileSync(fullPath, 'utf-8'));
    if (pkg.version !== targetVersion) {
      pkg.version = targetVersion;
      fs.writeFileSync(fullPath, JSON.stringify(pkg, null, 2) + '\n');
    }
  }
}

// 5. Sync apps/desktop/tauri.conf.json version
const tauriConfPath = path.join(rootDir, 'apps/desktop/tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'));
  tauriConf.version = targetVersion;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
}

console.log(`[Swifter] Synchronized all packages and configurations to version ${targetVersion}`);
