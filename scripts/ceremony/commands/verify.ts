// SPDX-License-Identifier: MIT
/**
 * ceremony verify <circuit-name>
 *
 * Checks a .zkey file against:
 *   1. The published Phase 1 transcript (via snarkjs zkey verify).
 *   2. The expected SHA-256 hash recorded in ceremony-manifest.json.
 *
 * Exit codes:
 *   0 — verification passed; keys are unmodified.
 *   1 — verification failed; a diff is printed to stderr.
 */

import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';
import { execSync } from 'child_process';

export interface VerifyOptions {
  circuitName: string;
  artifactsDir: string;
  manifestPath: string;
}

interface CeremonyManifest {
  generated_at: string;
  power: number;
  circuits: Record<string, string>;
}

/** Compute SHA-256 hex digest of a file. */
function sha256File(filePath: string): string {
  const data = fs.readFileSync(filePath);
  return crypto.createHash('sha256').update(data).digest('hex');
}

/** Run a shell command and return stdout. Returns null on non-zero exit. */
function runCapture(cmd: string): string | null {
  try {
    return execSync(cmd, { encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'] });
  } catch {
    return null;
  }
}

export async function runVerify(opts: VerifyOptions): Promise<number> {
  const { circuitName, artifactsDir, manifestPath } = opts;

  let passed = true;

  console.log(`\n🔍  Verifying ceremony artifacts for circuit: ${circuitName}\n`);

  // ── 1. Load manifest ────────────────────────────────────────────────────

  if (!fs.existsSync(manifestPath)) {
    console.error(`  Manifest not found: ${manifestPath}`);
    console.error('    Run `ceremony init` first, or obtain the published manifest.');
    return 1;
  }

  const manifest: CeremonyManifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));

  if (!(circuitName in manifest.circuits)) {
    console.error(`  Circuit "${circuitName}" not found in manifest.`);
    console.error(`    Available circuits: ${Object.keys(manifest.circuits).join(', ')}`);
    return 1;
  }

  const expectedHash = manifest.circuits[circuitName];

  // ── 2. Locate artifacts ─────────────────────────────────────────────────

  const zkeyPath = path.join(artifactsDir, `${circuitName}_final.zkey`);
  const ptauPath = path.join(artifactsDir, `pot${manifest.power}_final.ptau`);
  const r1csPath = path.join(artifactsDir, `${circuitName}.r1cs`);

  for (const [label, p] of [
    ['zkey', zkeyPath],
    ['ptau', ptauPath],
    ['r1cs', r1csPath],
  ] as [string, string][]) {
    if (!fs.existsSync(p)) {
      console.error(`  Missing artifact [${label}]: ${p}`);
      return 1;
    }
  }

  // ── 3. SHA-256 hash check ───────────────────────────────────────────────

  console.log('  [1/2] Checking SHA-256 hash against manifest…');
  const actualHash = sha256File(zkeyPath);

  if (actualHash === expectedHash) {
    console.log(`    Hash matches manifest.`);
    console.log(`      expected : ${expectedHash}`);
    console.log(`      actual   : ${actualHash}`);
  } else {
    console.error(`    Hash MISMATCH — .zkey may have been tampered with!`);
    console.error(`      expected : ${expectedHash}`);
    console.error(`      actual   : ${actualHash}`);
    console.error('\n  diff:');
    console.error(`  - ${expectedHash}  (manifest)`);
    console.error(`  + ${actualHash}  (file on disk)`);
    passed = false;
  }

  // ── 4. snarkjs zkey verification ────────────────────────────────────────

  console.log('\n  [2/2] Running snarkjs zkey verify against Phase 1 transcript…');
  const verifyCmd = `npx snarkjs zkey verify "${r1csPath}" "${ptauPath}" "${zkeyPath}"`;
  console.log(`  ▶ ${verifyCmd}`);

  const output = runCapture(verifyCmd);

  if (output !== null) {
    console.log(`    snarkjs zkey verify passed.`);
    if (output.trim()) {
      console.log(output.trim().split('\n').map((l: string) => `      ${l}`).join('\n'));
    }
  } else {
    console.error(`    snarkjs zkey verify FAILED.`);
    console.error('      The .zkey does not match the Phase 1 transcript.');
    passed = false;
  }

  // ── Result ──────────────────────────────────────────────────────────────

  console.log('');
  if (passed) {
    console.log(`  ceremony verify PASSED for circuit "${circuitName}"\n`);
    return 0;
  } else {
    console.error(`  ceremony verify FAILED for circuit "${circuitName}"\n`);
    return 1;
  }
}
