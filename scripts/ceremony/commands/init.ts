// SPDX-License-Identifier: MIT
/**
 * ceremony init — Powers of Tau (Phase 1) + circuit-specific setup (Phase 2).
 *
 * Wraps snarkjs ceremony commands with clear progress output and writes a
 * ceremony-manifest.json containing expected SHA-256 hashes of all .zkey files.
 */

import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';
import { execSync } from 'child_process';

export interface InitOptions {
  power: number;
  entropy?: string;
  outputDir: string;
  circuits: string[];
}

/** Compute SHA-256 hex digest of a file. */
function sha256File(filePath: string): string {
  const data = fs.readFileSync(filePath);
  return crypto.createHash('sha256').update(data).digest('hex');
}

/** Run a shell command and stream output to stdout/stderr. */
function run(cmd: string): void {
  console.log(`\n▶ ${cmd}`);
  execSync(cmd, { stdio: 'inherit' });
}

/** Print a section header. */
function section(title: string): void {
  console.log(`\n${'─'.repeat(60)}`);
  console.log(`  ${title}`);
  console.log('─'.repeat(60));
}

export async function runInit(opts: InitOptions): Promise<void> {
  const { power, entropy, outputDir, circuits } = opts;

  fs.mkdirSync(outputDir, { recursive: true });

  // ── Phase 1: Powers of Tau ──────────────────────────────────────────────

  section('Phase 1 — Powers of Tau');

  const ptauNew = path.join(outputDir, `pot${power}_0000.ptau`);
  const ptauBeacon = path.join(outputDir, `pot${power}_beacon.ptau`);
  const ptauFinal = path.join(outputDir, `pot${power}_final.ptau`);

  console.log(`[1/3] Initializing Powers of Tau (2^${power} constraints)…`);
  run(`npx snarkjs powersoftau new bn128 ${power} "${ptauNew}" -v`);

  console.log('\n[2/3] Applying randomness beacon (Ethereum block hash convention)…');
  const beaconHash = entropy
    ? crypto.createHash('sha256').update(entropy).digest('hex')
    : '0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20';
  run(
    `npx snarkjs powersoftau beacon "${ptauNew}" "${ptauBeacon}" ${beaconHash} 10 -n="Ceremony beacon"`,
  );

  console.log('\n[3/3] Preparing for Phase 2 (final Powers of Tau)…');
  run(`npx snarkjs powersoftau prepare phase2 "${ptauBeacon}" "${ptauFinal}" -v`);

  // Verify Phase 1
  section('Verifying Phase 1 transcript');
  run(`npx snarkjs powersoftau verify "${ptauFinal}"`);
  console.log('  Phase 1 verified.');

  // ── Phase 2: Circuit-specific setup ────────────────────────────────────

  section('Phase 2 — Circuit-specific setup');

  const manifest: Record<string, string> = {};

  for (const circuit of circuits) {
    console.log(`\n──  Circuit: ${circuit}`);

    const r1csPath = path.join(outputDir, `${circuit}.r1cs`);
    const zkey0 = path.join(outputDir, `${circuit}_0000.zkey`);
    const zkeyFinal = path.join(outputDir, `${circuit}_final.zkey`);
    const vkeyPath = path.join(outputDir, `${circuit}_verification_key.json`);

    if (!fs.existsSync(r1csPath)) {
      console.warn(`⚠  ${r1csPath} not found — skipping ${circuit}. Compile the circuit first.`);
      continue;
    }

    console.log('  [1/3] Initialising zkey…');
    run(`npx snarkjs groth16 setup "${r1csPath}" "${ptauFinal}" "${zkey0}"`);

    console.log('  [2/3] Contributing to Phase 2…');
    const contrib = entropy ?? 'default-ceremony-entropy-replace-in-production';
    run(
      `npx snarkjs zkey contribute "${zkey0}" "${zkeyFinal}" --name="ceremony-contributor" -e="${contrib}"`,
    );

    console.log('  [3/3] Exporting verification key…');
    run(`npx snarkjs zkey export verificationkey "${zkeyFinal}" "${vkeyPath}"`);

    // Record hash in manifest
    const hash = sha256File(zkeyFinal);
    manifest[circuit] = hash;
    console.log(`    ${circuit}_final.zkey  sha256=${hash}`);
  }

  // ── Write ceremony-manifest.json ────────────────────────────────────────

  section('Writing ceremony-manifest.json');

  const manifestPath = path.join(process.cwd(), 'ceremony-manifest.json');
  const manifestData = {
    generated_at: new Date().toISOString(),
    power,
    circuits: manifest,
  };

  fs.writeFileSync(manifestPath, JSON.stringify(manifestData, null, 2));
  console.log(`  Manifest written to ${manifestPath}`);
  console.log('\nPhase 1 + Phase 2 ceremony complete.\n');
}
