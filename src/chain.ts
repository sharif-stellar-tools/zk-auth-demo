/**
 * chain.ts
 *
 * Frontend SDK helper for submitting a snarkjs Groth16 proof to the
 * on-chain Soroban verifier contract.
 *
 * The helper serialises the snarkjs proof object into the 256-byte wire
 * format expected by verify_proof, then invokes the contract via the
 * Stellar JS SDK.
 *
 * Dependencies (add to package.json if not present):
 *   @stellar/stellar-sdk  ^13.x
 */

import {
  Contract,
  Networks,
  SorobanRpc,
  Transaction,
  TransactionBuilder,
  xdr,
  nativeToScVal,
  scValToNative,
  BASE_FEE,
} from '@stellar/stellar-sdk';

// ─── Types ────────────────────────────────────────────────────────────────────

/** Raw snarkjs groth16 proof object (as returned by snarkjs.groth16.fullProve). */
export interface Groth16Proof {
  pi_a: [string, string, string];
  pi_b: [[string, string], [string, string], [string, string]];
  pi_c: [string, string, string];
  protocol: 'groth16';
  curve: 'bn128';
}

export interface ChainVerifyResult {
  /** true = proof is valid on-chain, false = proof is invalid */
  valid: boolean;
  /** Ledger sequence number the result was confirmed in */
  ledger: number;
  /** Raw fee charged in stroops */
  feeCharged: number;
}

// ─── Serialisation ────────────────────────────────────────────────────────────

/**
 * Encode a decimal field element string as a 32-byte big-endian Buffer.
 * Throws if the value does not fit in 32 bytes.
 */
function fpToBytes(decimal: string): Buffer {
  const n = BigInt(decimal);
  const hex = n.toString(16).padStart(64, '0');
  if (hex.length > 64) throw new Error(`Field element overflows 32 bytes: ${decimal}`);
  return Buffer.from(hex, 'hex');
}

/**
 * Serialise a snarkjs Groth16 proof into the 256-byte wire format:
 *   pi_a  bytes [0..64]    G1 uncompressed (x ‖ y)
 *   pi_b  bytes [64..192]  G2 uncompressed (x1 ‖ x0 ‖ y1 ‖ y0)
 *   pi_c  bytes [192..256] G1 uncompressed (x ‖ y)
 *
 * The encoding is Ethereum-compatible (EIP-197 / big-endian).
 */
export function serializeProof(proof: Groth16Proof): Buffer {
  const pi_a = Buffer.concat([fpToBytes(proof.pi_a[0]), fpToBytes(proof.pi_a[1])]);

  // G2 Fp2 encoding: imaginary (index 1) before real (index 0)
  const pi_b = Buffer.concat([
    fpToBytes(proof.pi_b[0][1]), // x1
    fpToBytes(proof.pi_b[0][0]), // x0
    fpToBytes(proof.pi_b[1][1]), // y1
    fpToBytes(proof.pi_b[1][0]), // y0
  ]);

  const pi_c = Buffer.concat([fpToBytes(proof.pi_c[0]), fpToBytes(proof.pi_c[1])]);

  const result = Buffer.concat([pi_a, pi_b, pi_c]);
  if (result.length !== 256) throw new Error(`Proof serialized to ${result.length} bytes, expected 256`);
  return result;
}

/**
 * Encode public signals (decimal strings from snarkjs) as an array of
 * 32-byte XDR BytesN values for the public_inputs contract argument.
 */
function encodePublicInputs(publicSignals: string[]): xdr.ScVal {
  const items = publicSignals.map((s) => {
    const buf = fpToBytes(s);
    return xdr.ScVal.scvBytes(buf);
  });
  return xdr.ScVal.scvVec(items);
}

// ─── On-chain submission ──────────────────────────────────────────────────────

/**
 * Submit a Groth16 proof to the on-chain Soroban verifier and return
 * whether it is valid.
 *
 * @param proof         - snarkjs groth16 proof object
 * @param publicSignals - snarkjs publicSignals array (decimal strings)
 * @param contractId    - Soroban contract ID (C... address)
 * @param userAddress   - Stellar account address that owns the commitment
 * @param keypair       - Stellar Keypair for signing the transaction
 * @param options       - optional overrides for network / RPC URL
 */
export async function submitProofToChain(
  proof: Groth16Proof,
  publicSignals: string[],
  contractId: string,
  userAddress: string,
  keypair: { sign(data: Buffer): Buffer; publicKey(): string },
  options: {
    rpcUrl?: string;
    networkPassphrase?: string;
  } = {},
): Promise<ChainVerifyResult> {
  const rpcUrl = options.rpcUrl ?? 'https://soroban-testnet.stellar.org';
  const networkPassphrase = options.networkPassphrase ?? Networks.TESTNET;

  const server = new SorobanRpc.Server(rpcUrl, { allowHttp: false });
  const contract = new Contract(contractId);

  // Serialise proof to bytes
  const proofBytes = serializeProof(proof);
  const proofScVal = xdr.ScVal.scvBytes(proofBytes);

  // Encode public inputs
  const inputsScVal = encodePublicInputs(publicSignals);

  // Encode user address
  const userScVal = nativeToScVal(userAddress, { type: 'address' });

  // Build the transaction
  const account = await server.getAccount(keypair.publicKey());

  const tx: Transaction = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase,
  })
    .addOperation(
      contract.call('verify_proof', userScVal, proofScVal, inputsScVal),
    )
    .setTimeout(30)
    .build();

  // Simulate first to get the correct resource fee
  const simResult = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }

  const preparedTx = SorobanRpc.assembleTransaction(tx, simResult).build();

  // Sign
  preparedTx.sign(keypair as Parameters<Transaction['sign']>[0]);

  // Submit
  const sendResult = await server.sendTransaction(preparedTx);
  if (sendResult.status === 'ERROR') {
    throw new Error(`Transaction submission failed: ${JSON.stringify(sendResult.errorResult)}`);
  }

  // Poll for confirmation
  let getResult = await server.getTransaction(sendResult.hash);
  while (getResult.status === SorobanRpc.Api.GetTransactionStatus.NOT_FOUND) {
    await new Promise((r) => setTimeout(r, 1500));
    getResult = await server.getTransaction(sendResult.hash);
  }

  if (getResult.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
    throw new Error(`Transaction failed on-chain: ${sendResult.hash}`);
  }

  // Extract the bool return value
  const returnVal = getResult.returnValue;
  const valid = returnVal ? scValToNative(returnVal) === true : false;

  return {
    valid,
    ledger: getResult.ledger,
    feeCharged: Number(getResult.feeCharged ?? 0),
  };
}
