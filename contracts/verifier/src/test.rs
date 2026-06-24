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

    let result = client.verify(&user, &proof, &public_inputs);
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

    let result = client.verify(&user, &proof, &public_inputs);
    assert!(!result);
}
