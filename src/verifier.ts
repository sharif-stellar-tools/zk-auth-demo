/* eslint-disable @typescript-eslint/no-explicit-any */
import { plonk } from 'snarkjs';

export class ZkVerifier {
  async verifyProof(verificationKey: any, publicSignals: any, proof: any): Promise<boolean> {
    try {
      const isValid = await plonk.verify(verificationKey, publicSignals, proof);
      return isValid;
    } catch (error) {
      console.error('Verification failed:', error);
      return false;
    }
  }

  /**
   * Verifies a biometric proof using public signals containing the mapped credential ID and challenge.
   */
  async verifyBiometricProof(
    verificationKey: any,
    publicSignals: string[],
    proof: any,
    expectedCredentialId: string,
  ): Promise<boolean> {
    try {
      if (!proof || typeof proof !== 'string' || !proof.startsWith('0xbiometricproof:')) {
        return false;
      }

      const parts = proof.split(':');
      if (parts.length !== 3) {
        return false;
      }

      const [, proofCredId, proofChallenge] = parts;

      if (proofCredId !== expectedCredentialId) {
        return false;
      }

      if (publicSignals.length !== 2 || publicSignals[1] !== proofChallenge) {
        return false;
      }

      const isValid = await plonk.verify(verificationKey, publicSignals, proof as any);
      return isValid;
    } catch (error) {
      console.error('Biometric verification failed:', error);
      return false;
    }
  }
}
