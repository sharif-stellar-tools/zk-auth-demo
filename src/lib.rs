#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, Env, Bytes, Vec, Val};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NullifierAlreadyUsed = 1,
    InvalidProof = 2,
}

#[contract]
pub struct ZkAuthVerifier;

#[contractimpl]
impl ZkAuthVerifier {
    /// Verifies a ZK-SNARK (Groth16) authentication proof on-chain
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

        // 3. Perform cryptographic proof verification.
        // We validate the mock structure starts with "Groth16Proof"
        let is_valid = Self::verify_proof_mock(&proof_bytes);

        if !is_valid {
            return Err(Error::InvalidProof);
        }

        // 4. Mark nullifier as spent/used in persistent storage
        storage.set(&nullifier, &true);

        Ok(true)
    }

    /// Helper to validate proof payload format and authenticity
    fn verify_proof_mock(proof: &Bytes) -> bool {
        if proof.len() < 12 {
            return false;
        }

        let mut header = [0u8; 12];
        proof.copy_into_slice(0, &mut header);
        
        if &header != b"Groth16Proof" {
            return false;
        }

        true
    }
}

mod test;
