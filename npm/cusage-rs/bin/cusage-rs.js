#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');

function isMusl() {
  if (process.platform !== 'linux') {
    return false;
  }
  try {
    const report = process.report && process.report.getReport();
    if (report && report.header) {
      return !report.header.glibcVersionRuntime;
    }
  } catch (_) {
    // fall through to glibc default
  }
  return false;
}

// Platform packages are published under this scope (see npm/scripts/prepare-packages.js).
const SCOPE = '@psysician';

function candidatePackages() {
  const { platform, arch } = process;
  const base = `${SCOPE}/cusage-rs-${platform}-${arch}`;
  if (platform !== 'linux') {
    return [base];
  }
  // Prefer the exact libc match; the musl build is static and runs on glibc
  // systems too, so it is the safer cross-libc fallback.
  return isMusl() ? [`${base}-musl`, base] : [base, `${base}-musl`];
}

function resolveBinary() {
  const exe = process.platform === 'win32' ? 'cusage-rs.exe' : 'cusage-rs';
  for (const pkg of candidatePackages()) {
    try {
      return require.resolve(`${pkg}/bin/${exe}`);
    } catch (_) {
      // try next candidate
    }
  }
  console.error(
    `cusage-rs: no prebuilt binary installed for ${process.platform}-${process.arch}.\n` +
      'Supported platforms: linux, darwin, win32 on x64/arm64.\n' +
      'If your platform is supported, reinstall without --no-optional.\n' +
      'Alternatively: cargo install cusage-rs'
  );
  process.exit(1);
}

const result = spawnSync(resolveBinary(), process.argv.slice(2), { stdio: 'inherit' });
if (result.error) {
  console.error(`cusage-rs: failed to run binary: ${result.error.message}`);
  process.exit(1);
}
if (result.signal) {
  // Re-raise the same signal so callers observe the conventional 128+signal
  // exit status instead of a flattened 1.
  process.kill(process.pid, result.signal);
}
process.exit(result.status === null ? 1 : result.status);
