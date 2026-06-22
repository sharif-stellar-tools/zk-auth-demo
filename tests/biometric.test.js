import { jest } from '@jest/globals';

// Mock `snarkjs.groth16.verify` to control verification outcomes.
jest.unstable_mockModule('snarkjs', () => ({
  groth16: {
    verify: jest.fn(async (vk, publicSignals, proof) => {
      if (typeof proof === 'string' && proof.startsWith('0xbiometricproof:')) {
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

import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';

// Use dynamic import because tests run in ESM mode.
const { ZkProver } = await import('../src/prover.ts');
const { ZkVerifier } = await import('../src/verifier.ts');
const { WebAuthnManager } = await import('../src/core/webauthn.ts');

describe('Biometric data bridging into ZK circuits', () => {
  let prover;
  let verifier;
  const originalNavigator = global.navigator;
  const originalWindow = global.window;

  beforeAll(() => {
    prover = new ZkProver();
    verifier = new ZkVerifier();
  });

  afterAll(() => {
    // Restore globals
    Object.defineProperty(global, 'navigator', {
      value: originalNavigator,
      writable: true,
    });
    Object.defineProperty(global, 'window', {
      value: originalWindow,
      writable: true,
    });
  });

  describe('WebAuthn Environment Support', () => {
    it('should return false for isSupported when navigator.credentials is not defined', () => {
      Object.defineProperty(global, 'window', { value: undefined, configurable: true });
      expect(WebAuthnManager.isSupported()).toBe(false);
    });

    it('should throw an error on register when WebAuthn is not supported', async () => {
      Object.defineProperty(global, 'window', { value: undefined, configurable: true });
      await expect(WebAuthnManager.register('user', 'challenge')).rejects.toThrow(
        'WebAuthn is not supported in this environment'
      );
    });

    it('should throw an error on authenticate when WebAuthn is not supported', async () => {
      Object.defineProperty(global, 'window', { value: undefined, configurable: true });
      await expect(WebAuthnManager.authenticate('credentialId', 'challenge')).rejects.toThrow(
        'WebAuthn is not supported in this environment'
      );
    });
  });

  describe('WebAuthn Integration (Mocked Browser)', () => {
    beforeAll(() => {
      Object.defineProperty(global, 'window', {
        value: { location: { hostname: 'localhost' } },
        configurable: true,
      });

      const mockCreate = jest.fn().mockImplementation(async (options) => {
        return {
          id: 'mock-credential-id',
          rawId: new ArrayBuffer(8),
          type: 'public-key',
          response: {
            clientDataJSON: new ArrayBuffer(8),
          },
        };
      });

      const mockGet = jest.fn().mockImplementation(async (options) => {
        return {
          id: 'mock-credential-id',
          rawId: new ArrayBuffer(8),
          type: 'public-key',
          response: {
            clientDataJSON: new ArrayBuffer(8),
          },
        };
      });

      Object.defineProperty(global, 'navigator', {
        value: {
          credentials: {
            create: mockCreate,
            get: mockGet,
          },
        },
        configurable: true,
      });
    });

    it('should return true for isSupported when browser APIs are mocked', () => {
      expect(WebAuthnManager.isSupported()).toBe(true);
    });

    it('should successfully complete the WebAuthn registration flow', async () => {
      const cred = await WebAuthnManager.register('testuser', 'challenge-string');
      expect(cred).toBeDefined();
      expect(cred.id).toBe('mock-credential-id');
    });

    it('should successfully complete the WebAuthn authentication flow', async () => {
      const assertion = await WebAuthnManager.authenticate('6d6f636b2d63726564656e7469616c2d6964', 'challenge-string');
      expect(assertion).toBeDefined();
      expect(assertion.id).toBe('mock-credential-id');
    });
  });

  describe('ZK Proof & Biometric Mapping', () => {
    const credentialId = 'biometric-key-123';
    const challenge = 'random-challenge-456';

    it('should map credential ID deterministically to field elements', () => {
      const mapped1 = prover.mapCredentialIdToInputs(credentialId);
      const mapped2 = prover.mapCredentialIdToInputs(credentialId);
      
      expect(mapped1).toBe(mapped2);
      expect(typeof mapped1).toBe('string');
      
      const bigintVal = BigInt(mapped1);
      expect(bigintVal).toBeGreaterThan(0n);
      
      const prime = 21888242871839275222246405745257275088548364400416034343698204186575808495617n;
      expect(bigintVal).toBeLessThan(prime);
    });

    it('should generate a valid biometric proof', async () => {
      const { proof, publicSignals } = await prover.generateBiometricProof(credentialId, challenge);
      
      expect(proof).toContain('0xbiometricproof:');
      expect(proof).toContain(credentialId);
      expect(proof).toContain(challenge);
      
      expect(publicSignals.length).toBe(2);
      expect(publicSignals[0]).toBe(prover.mapCredentialIdToInputs(credentialId));
      expect(publicSignals[1]).toBe(challenge);
    });

    it('should verify a valid biometric proof (positive)', async () => {
      const { proof, publicSignals } = await prover.generateBiometricProof(credentialId, challenge);
      const vk = { dummy: 'vk' };
      
      const result = await verifier.verifyBiometricProof(vk, publicSignals, proof, credentialId);
      expect(result).toBe(true);
    });

    it('should reject proof if credential ID is tampered (negative)', async () => {
      const { proof, publicSignals } = await prover.generateBiometricProof(credentialId, challenge);
      const vk = { dummy: 'vk' };
      
      const result = await verifier.verifyBiometricProof(vk, publicSignals, proof, 'tampered-credential-id');
      expect(result).toBe(false);
    });

    it('should reject proof if public signals are tampered (negative)', async () => {
      const { proof, publicSignals } = await prover.generateBiometricProof(credentialId, challenge);
      const vk = { dummy: 'vk' };
      
      const tamperedSignals = [publicSignals[0], 'tampered-challenge'];
      
      const result = await verifier.verifyBiometricProof(vk, tamperedSignals, proof, credentialId);
      expect(result).toBe(false);
    });

    it('should reject proof if it does not have correct format', async () => {
      const vk = { dummy: 'vk' };
      const result = await verifier.verifyBiometricProof(vk, ['pub', 'challenge'], '0xproof', credentialId);
      expect(result).toBe(false);
    });
  });
});
