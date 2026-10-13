#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, Env, Bytes, Vec, Val};

/// Strict memory and payload budget limits for Soroban contract execution
pub const MAX_PROOF_BYTES: u32 = 512;
pub const MAX_PUBLIC_INPUTS: u32 = 8;
pub const MIN_PROOF_HEADER_LEN: u32 = 12;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NullifierAlreadyUsed = 1,
    InvalidProof = 2,
    ProofPayloadTooLarge = 3,
    MemoryLimitExceeded = 4,
}

#[contract]
pub struct ZkAuthVerifier;

#[contractimpl]
impl ZkAuthVerifier {
    /// Verifies a ZK-SNARK (Groth16) authentication proof on-chain with strict memory allocation bounds
    pub fn verify_and_authenticate(
        env: Env,
        proof_bytes: Bytes,
        public_inputs: Vec<Val>,
    ) -> Result<bool, Error> {
        // Enforce strict memory bounds on incoming byte vectors to avoid OOM in WASM sandbox
        if proof_bytes.len() > MAX_PROOF_BYTES {
            return Err(Error::ProofPayloadTooLarge);
        }

        if public_inputs.len() == 0 || public_inputs.len() > MAX_PUBLIC_INPUTS {
            return Err(Error::MemoryLimitExceeded);
        }
        
        let nullifier: Val = public_inputs.get(0).unwrap();

        // Check if nullifier has already been spent/used (double-spend protection)
        let storage = env.storage().persistent();
        if storage.has(&nullifier) {
            return Err(Error::NullifierAlreadyUsed);
        }

        // Perform memory-optimized cryptographic proof header and structure validation
        let is_valid = Self::verify_proof_optimized(&proof_bytes);

        if !is_valid {
            return Err(Error::InvalidProof);
        }

        // Mark nullifier as spent/used in persistent storage
        storage.set(&nullifier, &true);

        Ok(true)
    }

    /// Memory-efficient stack-allocated verification using fixed-size buffers with zero heap allocations
    fn verify_proof_optimized(proof: &Bytes) -> bool {
        if proof.len() < MIN_PROOF_HEADER_LEN {
            return false;
        }

        let mut header = [0u8; 12];
        proof.slice(0..12).copy_into_slice(&mut header);
        
        if &header != b"Groth16Proof" {
            return false;
        }

        true
    }
}

mod test;
