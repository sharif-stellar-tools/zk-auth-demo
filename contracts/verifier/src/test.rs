#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    vec, BytesN, Env, IntoVal, Vec,
};

fn create_config(env: &Env) -> Config {
    Config {
        key_hash: BytesN::from_array(env, &[0u8; 32]),
        circuit_type: 0,
        protocol_version: 1,
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let config = create_config(&env);
    let contract_id = env.register(VerifierContract, VerifierContractArgs::__constructor(&config));
    let client = VerifierContractClient::new(&env, &contract_id);

    let stored = client.get_config();
    assert_eq!(stored.key_hash, config.key_hash);
    assert_eq!(stored.circuit_type, config.circuit_type);
    assert_eq!(stored.protocol_version, config.protocol_version);
}

#[test]
fn test_register_and_verify() {
    let env = Env::default();
    let config = create_config(&env);
    let contract_id = env.register(VerifierContract, VerifierContractArgs::__constructor(&config));
    let client = VerifierContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register",
                args: (&user, &commitment).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register(&user, &commitment);

    let stored = client.get_commitment(&user);
    assert_eq!(stored, commitment);
}

#[test]
fn test_verify_with_matching_commitment() {
    let env = Env::default();
    let config = create_config(&env);
    let contract_id = env.register(VerifierContract, VerifierContractArgs::__constructor(&config));
    let client = VerifierContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register",
                args: (&user, &commitment).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register(&user, &commitment);

    let proof = BytesN::from_array(&env, &[0u8; 64]);
    let public_inputs: Vec<BytesN<32>> = vec![&env, commitment.clone()];
    let nullifier = BytesN::from_array(&env, &[42u8; 32]);

    let result = client.verify(&user, &proof, &public_inputs, &nullifier);
    assert!(result);
}

#[test]
fn test_verify_with_wrong_commitment() {
    let env = Env::default();
    let config = create_config(&env);
    let contract_id = env.register(VerifierContract, VerifierContractArgs::__constructor(&config));
    let client = VerifierContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register",
                args: (&user, &commitment).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register(&user, &commitment);

    let wrong = BytesN::from_array(&env, &[2u8; 32]);
    let proof = BytesN::from_array(&env, &[0u8; 64]);
    let public_inputs: Vec<BytesN<32>> = vec![&env, wrong];
    let nullifier = BytesN::from_array(&env, &[43u8; 32]);

    let result = client.verify(&user, &proof, &public_inputs, &nullifier);
    assert!(!result);
}

#[test]
fn test_nullifier_replay_rejected() {
    let env = Env::default();
    let config = create_config(&env);
    let contract_id = env.register(VerifierContract, VerifierContractArgs::__constructor(&config));
    let client = VerifierContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register",
                args: (&user, &commitment).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register(&user, &commitment);

    let proof = BytesN::from_array(&env, &[0u8; 64]);
    let public_inputs: Vec<BytesN<32>> = vec![&env, commitment.clone()];
    let nullifier = BytesN::from_array(&env, &[99u8; 32]);

    // First use succeeds.
    let result = client.verify(&user, &proof, &public_inputs, &nullifier);
    assert!(result);

    // Second use with the same nullifier must be rejected.
    let err = client
        .try_verify(&user, &proof, &public_inputs, &nullifier)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::NullifierAlreadyUsed);
}

#[test]
fn test_different_appid_produces_different_nullifier() {
    let env = Env::default();
    let config = create_config(&env);
    let contract_id = env.register(VerifierContract, VerifierContractArgs::__constructor(&config));
    let client = VerifierContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let commitment = BytesN::from_array(&env, &[1u8; 32]);

    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register",
                args: (&user, &commitment).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register(&user, &commitment);

    let proof = BytesN::from_array(&env, &[0u8; 64]);
    let public_inputs: Vec<BytesN<32>> = vec![&env, commitment.clone()];

    // Two distinct nullifiers simulate Poseidon(secret, appId1) vs Poseidon(secret, appId2).
    let nullifier_app1 = BytesN::from_array(&env, &[10u8; 32]);
    let nullifier_app2 = BytesN::from_array(&env, &[20u8; 32]);

    assert_ne!(nullifier_app1, nullifier_app2);

    let r1 = client.verify(&user, &proof, &public_inputs, &nullifier_app1);
    let r2 = client.verify(&user, &proof, &public_inputs, &nullifier_app2);
    assert!(r1);
    assert!(r2);
}
