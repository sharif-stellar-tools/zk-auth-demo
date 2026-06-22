/* eslint-disable @typescript-eslint/no-unused-vars */
export class ZkProver {
  async generateProof(_secret: string) {
    return '0xproof';
  }

  /**
   * Generates a proof utilizing WebAuthn credentials (biometrics) as input.
   */
  async generateBiometricProof(
    credentialId: string,
    challenge: string,
  ): Promise<{ proof: string; publicSignals: string[] }> {
    const mappedInput = this.mapCredentialIdToInputs(credentialId);
    return {
      proof: `0xbiometricproof:${credentialId}:${challenge}`,
      publicSignals: [mappedInput, challenge],
    };
  }

  /**
   * Maps a credential ID to a ZK circuit input field element.
   * Uses a rolling polynomial hash modulo the BN254 prime.
   */
  mapCredentialIdToInputs(credentialId: string): string {
    let hash = 0n;
    const prime = 21888242871839275222246405745257275088548364400416034343698204186575808495617n;
    for (let i = 0; i < credentialId.length; i++) {
      hash = (hash * 31n + BigInt(credentialId.charCodeAt(i))) % prime;
    }
    return hash.toString();
  }
}
