import { buildPoseidon } from 'circomlibjs';

let poseidon: Awaited<ReturnType<typeof buildPoseidon>> | null = null;

async function getPoseidon() {
  if (!poseidon) poseidon = await buildPoseidon();
  return poseidon;
}

/**
 * Computes a nullifier as Poseidon(secret, appId).
 * The result is a hex string suitable for passing as a public input.
 */
export async function computeNullifier(secret: bigint, appId: bigint): Promise<string> {
  const p = await getPoseidon();
  const hash = p([secret, appId]);
  const hex = p.F.toString(hash, 16).padStart(64, '0');
  return '0x' + hex;
}
