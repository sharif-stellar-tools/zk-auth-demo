// Export the ZK authentication button component
export { ZKLoginButton } from './components/ZKLoginButton.js';
export type { ZKLoginButtonProps } from './components/ZKLoginButton.js';

// Export existing ZK core logic
export { ZkProver } from './prover.js';
export { ZkVerifier } from './verifier.js';
export { WebAuthnManager } from './core/webauthn.js';

// On-chain Soroban verifier helpers
export { submitProofToChain, serializeProof } from './chain.js';
export type { Groth16Proof, ChainVerifyResult } from './chain.js';
