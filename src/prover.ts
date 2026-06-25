﻿import * as snarkjs from 'snarkjs';

import { ZkProofError, mapProverErrorToZkProofError } from './errors/zkErrors';

export interface ProofResult {
  proof: unknown;
  publicSignals: unknown;
}

export class ZkProver {
  async generateProof(
    input: Record<string, string | number | bigint>,
    wasmPath: string,
    zkeyPath: string,
  ): Promise<ProofResult> {
    if (!input || typeof input !== 'object') {
      throw new ZkProofError('ZK_INVALID_INPUT', 'Invalid input provided for proof generation.', {
        input,
      });
    }

    if (!wasmPath || !zkeyPath) {
      throw new ZkProofError(
        'ZK_MISSING_ARTIFACTS',
        'Missing circuit artifacts required to generate the proof.',
        { wasmPathPresent: Boolean(wasmPath), zkeyPathPresent: Boolean(zkeyPath) },
      );
    }

    try {
      const { proof, publicSignals } = (await snarkjs.groth16.fullProve(
        input,
        wasmPath,
        zkeyPath,
      )) as {
        proof: unknown;
        publicSignals: unknown;
      };

      return {
        proof,
        publicSignals,
      };
    } catch (error: unknown) {
      // Normalize all errors into a stable, user-safe error code/message.
      throw mapProverErrorToZkProofError(error);
    }
  }
}
