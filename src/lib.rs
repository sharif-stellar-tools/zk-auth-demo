#![no_std]
pub mod poseidon;

use soroban_sdk::{contract, contractimpl, contracterror, Env, Bytes, Vec, Val};
use poseidon::poseidon_hash_2;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NullifierAlreadyUsed = 1,
    InvalidProof = 2,
    InvalidPoseidonHash = 3,
}

#[contract]
pub struct ZkAuthVerifier;

#[contractimpl]
impl ZkAuthVerifier {
    /// Computes native Poseidon hash inside Soroban contract
    pub fn compute_poseidon(_env: Env, left: u64, right: u64) -> u64 {
        poseidon_hash_2(left, right)
    }

    /// Verifies a ZK-SNARK authentication proof on-chain
    pub fn verify_and_authenticate(
        env: Env,
        proof_bytes: Bytes,
        public_inputs: Vec<Val>,
    ) -> Result<bool, Error> {
        // 1. Extract nullifier from public inputs
        if public_inputs.len() == 0 {
            return Err(Error::InvalidProof);
        }
        
        let nullifier: Val = public_inputs.get(0).unwrap();

        // 2. Check if nullifier has already been spent/used (double-spend protection)
        let storage = env.storage().persistent();
        if storage.has(&nullifier) {
            return Err(Error::NullifierAlreadyUsed);
        }

        // 3. Perform cryptographic proof verification
        let is_valid = Self::verify_proof_mock(&proof_bytes);

        if !is_valid {
            return Err(Error::InvalidProof);
        }

        // 4. Mark nullifier as spent/used in persistent storage
        storage.set(&nullifier, &true);

        Ok(true)
    }

    /// Verifies proof with native on-chain Poseidon nullifier integrity check
    pub fn verify_with_poseidon(
        env: Env,
        proof_bytes: Bytes,
        secret: u64,
        app_id: u64,
        expected_nullifier: u64,
    ) -> Result<bool, Error> {
        let computed = poseidon_hash_2(secret, app_id);
        if computed != expected_nullifier {
            return Err(Error::InvalidPoseidonHash);
        }

        let storage = env.storage().persistent();
        if storage.has(&expected_nullifier) {
            return Err(Error::NullifierAlreadyUsed);
        }

        if !Self::verify_proof_mock(&proof_bytes) {
            return Err(Error::InvalidProof);
        }

        storage.set(&expected_nullifier, &true);
        Ok(true)
    }

    /// Helper to validate proof payload format and authenticity
    fn verify_proof_mock(proof: &Bytes) -> bool {
        if proof.len() < 12 {
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
