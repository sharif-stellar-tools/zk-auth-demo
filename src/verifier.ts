import { groth16 } from 'snarkjs';

export class ZkVerifier {
  async verifyProof(verificationKey: any, publicSignals: any, proof: any): Promise<boolean> {
    try {
      const isValid = await groth16.verify(verificationKey, publicSignals, proof);
      return isValid;
    } catch (error) {
      // Handle verification failures gracefully without breaking caller's control flow
      console.error('Verification failed:', error);
      return false;
    }
  }
}
