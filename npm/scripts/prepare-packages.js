#!/usr/bin/env node
// Generates the publishable npm packages (meta package + one package per
// platform) from release binaries. Run by the release workflow:
//
//   node npm/scripts/prepare-packages.js --version 1.1.0 --staging staging --out npm/dist
//
// Expects staging/<rust-target>/cusage-rs[.exe] for every target below and
// writes npm/dist/<package-name>/ directories ready for `npm publish`.
'use strict';

const fs = require('fs');
const path = require('path');

const TARGETS = {
  'x86_64-unknown-linux-gnu': { name: 'cusage-rs-linux-x64', os: 'linux', cpu: 'x64', libc: 'glibc' },
  'aarch64-unknown-linux-gnu': { name: 'cusage-rs-linux-arm64', os: 'linux', cpu: 'arm64', libc: 'glibc' },
  'x86_64-unknown-linux-musl': { name: 'cusage-rs-linux-x64-musl', os: 'linux', cpu: 'x64', libc: 'musl' },
  'aarch64-unknown-linux-musl': { name: 'cusage-rs-linux-arm64-musl', os: 'linux', cpu: 'arm64', libc: 'musl' },
  'x86_64-apple-darwin': { name: 'cusage-rs-darwin-x64', os: 'darwin', cpu: 'x64' },
  'aarch64-apple-darwin': { name: 'cusage-rs-darwin-arm64', os: 'darwin', cpu: 'arm64' },
  'x86_64-pc-windows-msvc': { name: 'cusage-rs-win32-x64', os: 'win32', cpu: 'x64' },
  'aarch64-pc-windows-msvc': { name: 'cusage-rs-win32-arm64', os: 'win32', cpu: 'arm64' },
};

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i];
    const value = argv[i + 1];
    if (!key.startsWith('--') || value === undefined) {
      throw new Error(`invalid arguments near ${key}`);
    }
    args[key.slice(2)] = value;
  }
  for (const required of ['version', 'staging', 'out']) {
    if (!args[required]) {
      throw new Error(`missing --${required}`);
    }
  }
  if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(args.version)) {
    throw new Error(`--version must be a semver version, got: ${args.version}`);
  }
  return args;
}

function writeJson(file, value) {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const repoRoot = path.resolve(__dirname, '..', '..');
  const metaSource = path.join(repoRoot, 'npm', 'cusage-rs');
  const license = path.join(repoRoot, 'LICENSE');
  const readme = path.join(repoRoot, 'README.md');

  fs.rmSync(args.out, { recursive: true, force: true });

  // Platform packages.
  for (const [target, spec] of Object.entries(TARGETS)) {
    const exe = spec.os === 'win32' ? 'cusage-rs.exe' : 'cusage-rs';
    const binary = path.join(args.staging, target, exe);
    if (!fs.existsSync(binary)) {
      throw new Error(`missing release binary: ${binary}`);
    }

    const pkgDir = path.join(args.out, spec.name);
    fs.mkdirSync(path.join(pkgDir, 'bin'), { recursive: true });
    fs.copyFileSync(binary, path.join(pkgDir, 'bin', exe));
    fs.chmodSync(path.join(pkgDir, 'bin', exe), 0o755);
    fs.copyFileSync(license, path.join(pkgDir, 'LICENSE'));

    const manifest = {
      name: spec.name,
      version: args.version,
      description: `cusage-rs binary for ${spec.os}-${spec.cpu}${spec.libc ? ` (${spec.libc})` : ''}`,
      license: 'MIT',
      repository: { type: 'git', url: 'git+https://github.com/Psysician/cusage-rs.git' },
      preferUnplugged: true,
      files: ['bin/'],
      os: [spec.os],
      cpu: [spec.cpu],
    };
    if (spec.libc) {
      manifest.libc = [spec.libc];
    }
    writeJson(path.join(pkgDir, 'package.json'), manifest);
  }

  // Meta package: committed template with version + optionalDependencies stamped.
  const metaDir = path.join(args.out, 'cusage-rs');
  fs.mkdirSync(path.join(metaDir, 'bin'), { recursive: true });
  fs.copyFileSync(
    path.join(metaSource, 'bin', 'cusage-rs.js'),
    path.join(metaDir, 'bin', 'cusage-rs.js')
  );
  fs.copyFileSync(license, path.join(metaDir, 'LICENSE'));
  fs.copyFileSync(readme, path.join(metaDir, 'README.md'));

  const meta = JSON.parse(fs.readFileSync(path.join(metaSource, 'package.json'), 'utf8'));
  meta.version = args.version;
  const expected = Object.values(TARGETS).map((spec) => spec.name);
  const declared = Object.keys(meta.optionalDependencies || {});
  if (expected.length !== declared.length || !expected.every((name) => declared.includes(name))) {
    throw new Error('optionalDependencies in npm/cusage-rs/package.json are out of sync with TARGETS');
  }
  for (const name of declared) {
    meta.optionalDependencies[name] = args.version;
  }
  writeJson(path.join(metaDir, 'package.json'), meta);

  console.log(`prepared ${expected.length + 1} packages at version ${args.version} in ${args.out}`);
}

main();
