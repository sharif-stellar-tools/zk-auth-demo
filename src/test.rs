#![cfg(test)]

use super::{ZkAuthVerifier, ZkAuthVerifierClient, Error};
use soroban_sdk::{Env, Bytes, Vec, Val};

#[test]
fn test_verification_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZkAuthVerifier);
    let client = ZkAuthVerifierClient::new(&env, &contract_id);

    // 1. Prepare valid mock proof bytes starting with "Groth16Proof"
    let mut proof_bytes = Bytes::new(&env);
    proof_bytes.append(&Bytes::from_slice(&env, b"Groth16Proofabcdef1234567890"));

    // 2. Prepare public inputs (nullifier hash as element 0)
    let nullifier = Val::from_i32(1001);
    let mut public_inputs: Vec<Val> = Vec::new(&env);
    public_inputs.push_back(nullifier.into());

    // 3. Call verify_and_authenticate - should pass and return true
    let result = client.verify_and_authenticate(&proof_bytes, &public_inputs);
    assert_eq!(result, true);

    // 4. Call verify_and_authenticate again with same nullifier - should fail (NullifierAlreadyUsed)
    let second_result = client.try_verify_and_authenticate(&proof_bytes, &public_inputs);
    assert_eq!(second_result, Err(Ok(Error::NullifierAlreadyUsed)));

    // 5. Call verify_and_authenticate with invalid proof bytes - should fail (InvalidProof)
    let invalid_proof = Bytes::from_slice(&env, b"InvalidHeader123");
    let new_nullifier = Val::from_i32(1002);
    let mut new_public_inputs: Vec<Val> = Vec::new(&env);
    new_public_inputs.push_back(new_nullifier.into());

    let third_result = client.try_verify_and_authenticate(&invalid_proof, &new_public_inputs);
    assert_eq!(third_result, Err(Ok(Error::InvalidProof)));
}

#[test]
fn test_native_poseidon_hash_and_verification() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ZkAuthVerifier);
    let client = ZkAuthVerifierClient::new(&env, &contract_id);

    let secret = 42u64;
    let app_id = 100u64;

    // 1. Compute native Poseidon hash
    let hash = client.compute_poseidon(&secret, &app_id);
    assert_ne!(hash, 0);

    // Deterministic verify
    let hash2 = client.compute_poseidon(&secret, &app_id);
    assert_eq!(hash, hash2);

    // 2. Verify with correct Poseidon hash
    let proof = Bytes::from_slice(&env, b"Groth16Proof_valid_zk_payload");
    let verified = client.verify_with_poseidon(&proof, &secret, &app_id, &hash);
    assert_eq!(verified, true);

    // 3. Second call with same hash fails (NullifierAlreadyUsed)
    let second_call = client.try_verify_with_poseidon(&proof, &secret, &app_id, &hash);
    assert_eq!(second_call, Err(Ok(Error::NullifierAlreadyUsed)));

    // 4. Call with wrong Poseidon hash fails (InvalidPoseidonHash)
    let wrong_hash = hash + 1;
    let wrong_call = client.try_verify_with_poseidon(&proof, &secret, &app_id, &wrong_hash);
    assert_eq!(wrong_call, Err(Ok(Error::InvalidPoseidonHash)));
}
