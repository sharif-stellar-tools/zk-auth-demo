#!/usr/bin/env npx ts-node
/**
 * gen-vkey-hash.ts
 *
 * Converts a snarkjs verification_key.json into the flat Vec<BytesN<32>>
 * encoding expected by the on-chain Soroban verifier constructor.
 *
 * Usage:
 *   npx ts-node scripts/gen-vkey-hash.ts <verification_key.json>
 *
 * Output:
 *   A JSON array of 32-byte hex strings (the vkey_chunks argument for
 *   the __constructor call), plus the SHA-256 of the whole blob for
 *   reference.
 *
 * Vkey chunk layout (matches lib.rs constants):
 *   [0..2]   alpha_g1   (G1, 2 × 32 bytes)
 *   [2..6]   beta_g2    (G2, 4 × 32 bytes)
 *   [6..10]  gamma_g2   (G2, 4 × 32 bytes)
 *   [10..14] delta_g2   (G2, 4 × 32 bytes)
 *   [14..16] IC[0]      (G1, 2 × 32 bytes)
 *   [16..18] IC[1]      (G1, 2 × 32 bytes)
 *   ...
 *
 * snarkjs vkey format reference:
 *   https://github.com/iden3/snarkjs#verification-key
 */

import { createHash } from 'crypto';
import { readFileSync } from 'fs';

// ─── Types ────────────────────────────────────────────────────────────────────

interface SnarkjsVKey {
  protocol: string;
  curve: string;
  nPublic: number;
  vk_alpha_1: [string, string, string];
  vk_beta_2: [[string, string], [string, string], [string, string]];
  vk_gamma_2: [[string, string], [string, string], [string, string]];
  vk_delta_2: [[string, string], [string, string], [string, string]];
  IC: [string, string, string][];
}

// ─── Serialisation helpers ────────────────────────────────────────────────────

/** Encode a decimal field element string as a 32-byte big-endian hex string. */
function fpToHex(decimal: string): string {
  const n = BigInt(decimal);
  const hex = n.toString(16).padStart(64, '0');
  if (hex.length > 64) throw new Error(`Field element too large: ${decimal}`);
  return hex;
}

/** Serialise a G1 affine point → two 32-byte chunks [x, y]. */
function g1ToChunks(point: [string, string, string]): string[] {
  // snarkjs G1 format: [x, y, z] projective; z should be "1" for affine.
  return [fpToHex(point[0]), fpToHex(point[1])];
}

/**
 * Serialise a G2 affine point → four 32-byte chunks [x1, x0, y1, y0].
 *
 * snarkjs G2 format: [[x1, x0], [y1, y0], [z1, z0]]
 * Ethereum / Soroban encoding:  x1 ‖ x0 ‖ y1 ‖ y0  (c1 before c0 per EIP-197)
 */
function g2ToChunks(point: [[string, string], [string, string], [string, string]]): string[] {
  return [
    fpToHex(point[0][1]), // x1 (imaginary)
    fpToHex(point[0][0]), // x0 (real)
    fpToHex(point[1][1]), // y1 (imaginary)
    fpToHex(point[1][0]), // y0 (real)
  ];
}

// ─── Main ─────────────────────────────────────────────────────────────────────

function main(): void {
  const vkeyPath = process.argv[2];
  if (!vkeyPath) {
    console.error('Usage: npx ts-node scripts/gen-vkey-hash.ts <verification_key.json>');
    process.exit(1);
  }

  const vkey: SnarkjsVKey = JSON.parse(readFileSync(vkeyPath, 'utf8'));

  if (vkey.protocol !== 'groth16') {
    console.error(`Expected groth16 protocol, got: ${vkey.protocol}`);
    process.exit(1);
  }
  if (vkey.curve !== 'bn128') {
    console.error(`Expected bn128 curve, got: ${vkey.curve}`);
    process.exit(1);
  }

  const chunks: string[] = [
    ...g1ToChunks(vkey.vk_alpha_1),
    ...g2ToChunks(vkey.vk_beta_2),
    ...g2ToChunks(vkey.vk_gamma_2),
    ...g2ToChunks(vkey.vk_delta_2),
    ...vkey.IC.flatMap((ic) => g1ToChunks(ic)),
  ];

  // SHA-256 of the concatenated raw bytes (for human reference / audit log)
  const raw = Buffer.concat(chunks.map((h) => Buffer.from(h, 'hex')));
  const sha256 = createHash('sha256').update(raw).digest('hex');

  const output = {
    nPublic: vkey.nPublic,
    chunkCount: chunks.length,
    sha256,
    // Array of 0x-prefixed 32-byte hex strings ready to paste into a deploy script
    vkeyChunks: chunks.map((h) => `0x${h}`),
  };

  console.log(JSON.stringify(output, null, 2));
}

main();
