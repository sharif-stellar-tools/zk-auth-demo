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
    proof_bytes.append(b"Groth16Proof");
    proof_bytes.append(b"abcdef1234567890");

    // 2. Prepare public inputs (nullifier hash as element 0)
    let nullifier = Val::from_i32(1001);
    let mut public_inputs = Vec::new(&env);
    public_inputs.push_back(nullifier);

    // 3. Call verify_and_authenticate - should pass and return true
    let result = client.verify_and_authenticate(&proof_bytes, &public_inputs);
    assert_eq!(result, true);

    // 4. Call verify_and_authenticate again with same nullifier - should fail (NullifierAlreadyUsed)
    let second_result = client.try_verify_and_authenticate(&proof_bytes, &public_inputs);
    assert_eq!(second_result, Err(Ok(Error::NullifierAlreadyUsed)));

    // 5. Call verify_and_authenticate with invalid proof bytes - should fail (InvalidProof)
    let invalid_proof = Bytes::from_slice(&env, b"InvalidHeader123");
    let new_nullifier = Val::from_i32(1002);
    let mut new_public_inputs = Vec::new(&env);
    new_public_inputs.push_back(new_nullifier);

    let third_result = client.try_verify_and_authenticate(&invalid_proof, &new_public_inputs);
    assert_eq!(third_result, Err(Ok(Error::InvalidProof)));
}
