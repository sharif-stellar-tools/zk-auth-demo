import { jest } from '@jest/globals';

// Mock `snarkjs.groth16.verify` to control verification outcomes.
// The mock returns `true` only when the proof and public signals match expected values.
jest.unstable_mockModule('snarkjs', () => ({
  groth16: {
    verify: jest.fn(async (vk, publicSignals, proof) => {
      if (proof === '0xproof' && JSON.stringify(publicSignals) === JSON.stringify(['pub'])) return true;
      return false;
    }),
  },
}));

// Use dynamic import because tests run in ESM mode.
const { ZkProver } = await import('../src/prover.ts');
const { ZkVerifier } = await import('../src/verifier.ts');
const snarkjs = await import('snarkjs');

describe('ZK proof generation & verification', () => {
  let prover;
  let verifier;

  beforeAll(() => {
    prover = new ZkProver();
    verifier = new ZkVerifier();
  });

  test('valid proof is verified (positive)', async () => {
    const secret = 'my-secret';
    const proof = await prover.generateProof(secret);

    const vk = { dummy: 'vk' };
    const publicSignals = ['pub'];

    const result = await verifier.verifyProof(vk, publicSignals, proof);

    expect(result).toBe(true);
    expect(snarkjs.groth16.verify).toHaveBeenCalledWith(vk, publicSignals, proof);
  });

  test('tampered public signals cause verification to fail (negative)', async () => {
    const secret = 'my-secret';
    const proof = await prover.generateProof(secret);

    const vk = { dummy: 'vk' };
    const publicSignals = ['pub'];

    // Simulate tampering: change the public inputs used for verification
    // after the proof was generated. Here we pass `tamperedPublicSignals`
    // to `verifyProof` to simulate an attacker altering the public inputs.
    const tamperedPublicSignals = ['pub-tampered'];

    const result = await verifier.verifyProof(vk, tamperedPublicSignals, proof);

    // The mocked `snarkjs.groth16.verify` returns false for the tampered inputs.
    expect(result).toBe(false);
  });
});
