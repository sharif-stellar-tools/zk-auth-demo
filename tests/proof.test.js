import { jest } from '@jest/globals';

// Mock snarkjs.plonk.verify to control verification outcomes.
// The mock returns 	rue only when the proof and public signals match expected values.
jest.unstable_mockModule('snarkjs', () => ({
  plonk: {
    verify: jest.fn(async (vk, publicSignals, proof) => {
      if (proof === '0xplonkproof' && JSON.stringify(publicSignals) === JSON.stringify(['pub'])) return true;
      if (typeof proof === 'string' && proof.startsWith('0xplonkbiometricproof:')) {
        const parts = proof.split(':');
        if (parts.length === 3) {
          const [, credentialId, challenge] = parts;
          let hash = 0n;
          const prime = 21888242871839275222246405745257275088548364400416034343698204186575808495617n;
          for (let i = 0; i < credentialId.length; i++) {
            hash = (hash * 31n + BigInt(credentialId.charCodeAt(i))) % prime;
          }
          if (
            publicSignals &&
            publicSignals[0] === hash.toString() &&
            publicSignals[1] === challenge
          ) {
            return true;
          }
        }
      }
      return false;
    }),
  },
}));


// Use dynamic import because tests run in ESM mode.
const { ZkProver } = await import('../src/prover.ts');
const { ZkVerifier } = await import('../src/verifier.ts');
const snarkjs = await import('snarkjs');

describe('ZK proof generation & verification (Plonk)', () => {
  let prover;
  let verifier;

  beforeAll(() => {
    prover = new ZkProver();
    verifier = new ZkVerifier();
  });

  test('valid Plonk proof is verified (positive)', async () => {
    const secret = 'my-secret';
    const proof = await prover.generateProof(secret);

    const vk = { dummy: 'vk' };
    const publicSignals = ['pub'];

    const result = await verifier.verifyProof(vk, publicSignals, proof);

    expect(result).toBe(true);
    expect(snarkjs.plonk.verify).toHaveBeenCalledWith(vk, publicSignals, proof);
  });

  test('tampered public signals cause Plonk verification to fail (negative)', async () => {
    const secret = 'my-secret';
    const proof = await prover.generateProof(secret);

    const vk = { dummy: 'vk' };
    const publicSignals = ['pub'];
    const tamperedPublicSignals = ['pub-tampered'];

    const result = await verifier.verifyProof(vk, tamperedPublicSignals, proof);

    expect(result).toBe(false);
  });

  test('Plonk proof format uses plonk prefix', async () => {
    const secret = 'my-secret';
    const proof = await prover.generateProof(secret);
    expect(proof).toBe('0xplonkproof');
  });

  test('biometric proof uses plonk format', async () => {
    const proof = await prover.generateBiometricProof('cred123', 'challenge456');
    expect(proof.proof).toMatch(/^0xplonkbiometricproof:/);
    expect(proof.publicSignals).toHaveLength(2);
  });
});
