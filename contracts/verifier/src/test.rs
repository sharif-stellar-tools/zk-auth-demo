#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    vec, Bytes, BytesN, Env, IntoVal, Vec,
};

// ─── Test vkey helpers ────────────────────────────────────────────────────────
//
// For unit tests we don't have a real circuit, so we build a fake-but-structurally-
// valid vkey and a proof that satisfies the pairing equation exactly.
//
// The trick: use the BN254 generator G1 and G2 as all vkey points and set
// pi_a = G1, pi_b = G2, pi_c = G1, public inputs all zero.
// The pairing check will NOT pass (that's fine — we test that the contract
// returns false for this, and true only for a proof we craft to pass).
//
// For the "valid proof" path we test the structural / commitment checks
// independently of the pairing, because in Soroban's test environment the
// pairing_check host function is available and runs real BN254 arithmetic —
// we can therefore supply a real proof/vkey pair.
//
// Known-good BN254 test vector from EIP-197 test suite:
//   https://github.com/ethereum/EIPs/blob/master/EIPS/eip-197.md
// This is the trivial "1 == 1" Groth16 proof where:
//   vkey = (G1, G2, G2, G2, G1, G1)   (alpha, beta, gamma, delta, IC[0], IC[1])
//   proof = (-G1, G2, G1)              pi_a negated so pairing_check == 1
//   public_input = [0]                 one input, scalar 0 → IC[1] * 0 = 0, vk_x = IC[0]
//
// We use a 2-public-input circuit (commitment + nonce) to match the real circuit.
// For structural/rejection tests we don't need the pairing to pass.

// BN254 G1 generator (uncompressed, big-endian, Ethereum format)
const G1_GEN: [u8; 64] = [
    // x = 1
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    // y = 2
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
];

// BN254 -G1 (negated generator): same x, y = p - 2
// p = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47
// p - 2 = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd45
const G1_NEG_GEN: [u8; 64] = [
    // x = 1
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    // y = p - 2
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29,
    0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d,
    0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c, 0xfd, 0x45,
];

// BN254 G2 generator (uncompressed, big-endian, Ethereum format)
// x = (x1, x0), y = (y1, y0)  — Fp2 element encoding: c1 || c0
const G2_GEN: [u8; 128] = [
    // x1
    0x19, 0x8e, 0x93, 0x93, 0x92, 0x0d, 0x48, 0x3a,
    0x72, 0x60, 0xbf, 0xb7, 0x31, 0xfb, 0x5d, 0x25,
    0xf1, 0xaa, 0x49, 0x33, 0x35, 0xa9, 0xe7, 0x12,
    0x97, 0xe4, 0x85, 0xb7, 0xae, 0xf3, 0x12, 0xc2,
    // x0
    0x18, 0x00, 0xde, 0xef, 0x12, 0x1f, 0x1e, 0x76,
    0x42, 0x6a, 0x00, 0x66, 0x5e, 0x5c, 0x44, 0x79,
    0x67, 0x4d, 0x86, 0x11, 0x95, 0xe0, 0x52, 0x19,
    0x17, 0x1d, 0x76, 0x30, 0xf2, 0x23, 0xb5, 0x86,
    // y1
    0x09, 0x06, 0x89, 0xd0, 0x58, 0x5f, 0xf0, 0x75,
    0xec, 0x9e, 0x99, 0xad, 0x69, 0x0c, 0x33, 0x95,
    0xbc, 0x4b, 0x31, 0x33, 0x70, 0xb3, 0x8e, 0xf3,
    0x55, 0xac, 0xda, 0xdc, 0xd1, 0x22, 0x97, 0x5b,
    // y0
    0x12, 0xc8, 0x5e, 0xa5, 0xdb, 0x8c, 0x6d, 0xeb,
    0x4a, 0xab, 0x71, 0x80, 0x8d, 0xcb, 0x40, 0x8f,
    0xe3, 0xd1, 0xe7, 0x69, 0x0c, 0x43, 0xd3, 0x7b,
    0x4c, 0xe6, 0xcc, 0x01, 0x66, 0xfa, 0x7d, 0xaa,
];

// ─── Contract setup ───────────────────────────────────────────────────────────

/// Build a minimal valid 2-input vkey using the BN254 generator points.
/// alpha=G1, beta=G2, gamma=G2, delta=G2, IC[0]=G1, IC[1]=G1, IC[2]=G1
fn make_vkey(env: &Env) -> Vec<BytesN<32>> {
    let mut chunks: Vec<BytesN<32>> = Vec::new(env);
    // alpha (G1 → 2 chunks)
    push_g1_chunks(env, &mut chunks, &G1_GEN);
    // beta, gamma, delta (G2 → 4 chunks each)
    push_g2_chunks(env, &mut chunks, &G2_GEN);
    push_g2_chunks(env, &mut chunks, &G2_GEN);
    push_g2_chunks(env, &mut chunks, &G2_GEN);
    // IC[0], IC[1], IC[2] (G1 → 2 chunks each)
    push_g1_chunks(env, &mut chunks, &G1_GEN);
    push_g1_chunks(env, &mut chunks, &G1_GEN);
    push_g1_chunks(env, &mut chunks, &G1_GEN);
    chunks
}

fn push_g1_chunks(env: &Env, chunks: &mut Vec<BytesN<32>>, g1: &[u8; 64]) {
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    a.copy_from_slice(&g1[0..32]);
    b.copy_from_slice(&g1[32..64]);
    chunks.push_back(BytesN::from_array(env, &a));
    chunks.push_back(BytesN::from_array(env, &b));
}

fn push_g2_chunks(env: &Env, chunks: &mut Vec<BytesN<32>>, g2: &[u8; 128]) {
    for i in 0..4usize {
        let mut c = [0u8; 32];
        c.copy_from_slice(&g2[i * 32..(i + 1) * 32]);
        chunks.push_back(BytesN::from_array(env, &c));
    }
}

fn make_config() -> Config {
    Config { public_input_count: 2, protocol_version: 1 }
}

fn deploy(env: &Env) -> (soroban_sdk::Address, VerifierContractClient) {
    let vkey = make_vkey(env);
    let config = make_config();
    let contract_id = env.register(
        VerifierContract,
        VerifierContractArgs::__constructor(&config, &vkey),
    );
    let client = VerifierContractClient::new(env, &contract_id);
    (contract_id, client)
}

fn register_user(
    env: &Env,
    contract_id: &soroban_sdk::Address,
    client: &VerifierContractClient,
    commitment: &BytesN<32>,
) -> soroban_sdk::Address {
    let user = soroban_sdk::Address::generate(env);
    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "register",
                args: (&user, commitment).into_val(env),
                sub_invokes: &[],
            },
        }])
        .register(&user, commitment);
    user
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_constructor_stores_config() {
    let env = Env::default();
    let (_, client) = deploy(&env);
    let cfg = client.get_config();
    assert_eq!(cfg.public_input_count, 2);
    assert_eq!(cfg.protocol_version, 1);
}

#[test]
#[should_panic]
fn test_double_init_panics() {
    let env = Env::default();
    let vkey = make_vkey(&env);
    let config = make_config();
    let contract_id = env.register(
        VerifierContract,
        VerifierContractArgs::__constructor(&config, &vkey),
    );
    let client = VerifierContractClient::new(&env, &contract_id);
    // second call must panic with AlreadyInitialized
    client.__constructor(&config, &vkey);
}

#[test]
fn test_register_stores_commitment() {
    let env = Env::default();
    let (contract_id, client) = deploy(&env);
    let commitment = BytesN::from_array(&env, &[0xab; 32]);
    let user = register_user(&env, &contract_id, &client, &commitment);
    assert_eq!(client.get_commitment(&user), commitment);
}

#[test]
#[should_panic]
fn test_double_register_panics() {
    let env = Env::default();
    let (contract_id, client) = deploy(&env);
    let commitment = BytesN::from_array(&env, &[0x01; 32]);
    let user = register_user(&env, &contract_id, &client, &commitment);
    // second register for same user must panic with CommitmentExists
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
}

#[test]
fn test_wrong_proof_length_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = deploy(&env);
    let commitment = BytesN::from_array(&env, &[0x01; 32]);
    let user = register_user(&env, &contract_id, &client, &commitment);

    // proof is 64 bytes instead of 256 — structurally invalid
    let short_proof = Bytes::from_array(&env, &[0u8; 64]);
    let inputs: Vec<BytesN<32>> = vec![
        &env,
        commitment.clone(),
        BytesN::from_array(&env, &[0u8; 32]),
    ];

    let result = client.verify_proof(&user, &short_proof, &inputs);
    assert!(!result, "short proof should return false, not panic");
}

#[test]
fn test_wrong_commitment_in_inputs_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = deploy(&env);
    let commitment = BytesN::from_array(&env, &[0x01; 32]);
    let user = register_user(&env, &contract_id, &client, &commitment);

    let proof = Bytes::from_array(&env, &[0u8; 256]);
    // supply a different value in public_inputs[0] — mismatch with stored commitment
    let wrong = BytesN::from_array(&env, &[0x99; 32]);
    let inputs: Vec<BytesN<32>> = vec![
        &env,
        wrong,
        BytesN::from_array(&env, &[0u8; 32]),
    ];

    let result = client.verify_proof(&user, &proof, &inputs);
    assert!(!result, "commitment mismatch should return false");
}

#[test]
fn test_wrong_input_count_returns_false() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = deploy(&env);
    let commitment = BytesN::from_array(&env, &[0x01; 32]);
    let user = register_user(&env, &contract_id, &client, &commitment);

    let proof = Bytes::from_array(&env, &[0u8; 256]);
    // only one input instead of two
    let inputs: Vec<BytesN<32>> = vec![&env, commitment.clone()];

    let result = client.verify_proof(&user, &proof, &inputs);
    assert!(!result, "wrong input count should return false");
}

#[test]
fn test_all_zeros_proof_returns_false() {
    // A 256-byte all-zero proof is structurally valid (passes length check)
    // but will fail the pairing check because zero-bytes don't encode valid
    // curve points for a real vkey.
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = deploy(&env);
    let commitment = BytesN::from_array(&env, &[0x01; 32]);
    let user = register_user(&env, &contract_id, &client, &commitment);

    let proof = Bytes::from_array(&env, &[0u8; 256]);
    let inputs: Vec<BytesN<32>> = vec![
        &env,
        commitment.clone(),
        BytesN::from_array(&env, &[0u8; 32]),
    ];

    // Should return false (bad points in pairing) without panicking.
    let result = client.verify_proof(&user, &proof, &inputs);
    assert!(!result, "invalid curve points should return false");
}

#[test]
fn test_valid_eip197_proof_passes() {
    // EIP-197 test vector: e(G1, G2) * e(-G1, G2) == 1
    // We construct a Groth16 proof that trivially satisfies the pairing equation:
    //
    //   vkey: alpha=G1, beta=G2, gamma=G2, delta=G2, IC[0]=G1, IC[1]=0, IC[2]=0
    //   proof: pi_a = -G1 (negated), pi_b = G2, pi_c = 0 (point at infinity)
    //   public_inputs: [0, 0]
    //
    // The pairing check becomes:
    //   e(-(-G1), G2) * e(G1, G2) * e(G1 + 0*G1 + 0*G1, G2) * e(0, G2)
    // = e(G1, G2) * e(G1, G2) * e(G1, G2) * 1
    // That isn't 1. Instead we use the standard EIP-197 trivial pair:
    //   [G1, G2], [-G1, G2]  → e(G1,G2)*e(-G1,G2) = 1
    //
    // To make the full Groth16 check pass we need to set up the vkey so that
    // the four-pair product equals 1. The cleanest way is:
    //   pi_a = G1, pi_b = G2 (so e(-pi_a, pi_b) = e(-G1, G2))
    //   alpha = G1, beta = G2 (so e(alpha, beta) = e(G1, G2))
    //   vk_x = 0 (point at infinity, so e(vk_x, gamma) = 1)
    //   pi_c = 0 (point at infinity, so e(pi_c, delta) = 1)
    //
    //   Product = e(-G1,G2) * e(G1,G2) * 1 * 1 = 1  ✓
    //
    // For vk_x = IC[0] + inputs[0]*IC[1] + inputs[1]*IC[2] to be the point
    // at infinity, we set IC[0]=0, IC[1]=0, IC[2]=0.

    let env = Env::default();
    env.mock_all_auths();

    let zero_g1 = [0u8; 64]; // point at infinity in G1
    let zero_g2 = [0u8; 128]; // point at infinity in G2

    let mut vkey_chunks: Vec<BytesN<32>> = Vec::new(&env);
    // alpha = G1 gen
    push_g1_chunks(&env, &mut vkey_chunks, &G1_GEN);
    // beta = G2 gen
    push_g2_chunks(&env, &mut vkey_chunks, &G2_GEN);
    // gamma = G2 gen (any valid G2; e(0, gamma) = 1)
    push_g2_chunks(&env, &mut vkey_chunks, &G2_GEN);
    // delta = G2 gen (any valid G2; e(0, delta) = 1)
    push_g2_chunks(&env, &mut vkey_chunks, &G2_GEN);
    // IC[0] = 0, IC[1] = 0, IC[2] = 0  → vk_x = 0
    push_g1_chunks(&env, &mut vkey_chunks, &zero_g1);
    push_g1_chunks(&env, &mut vkey_chunks, &zero_g1);
    push_g1_chunks(&env, &mut vkey_chunks, &zero_g1);

    let config = Config { public_input_count: 2, protocol_version: 1 };
    let contract_id = env.register(
        VerifierContract,
        VerifierContractArgs::__constructor(&config, &vkey_chunks),
    );
    let client = VerifierContractClient::new(&env, &contract_id);

    // commitment = 0x00..00 (matches IC structure — inputs don't matter when IC=0)
    let commitment = BytesN::from_array(&env, &[0u8; 32]);
    let user = register_user(&env, &contract_id, &client, &commitment);

    // proof: pi_a = -G1, pi_b = G2, pi_c = 0
    let mut proof_bytes = [0u8; 256];
    proof_bytes[0..64].copy_from_slice(&G1_NEG_GEN);
    proof_bytes[64..192].copy_from_slice(&G2_GEN);
    // pi_c stays zero (point at infinity)

    let proof = Bytes::from_array(&env, &proof_bytes);
    let inputs: Vec<BytesN<32>> = vec![
        &env,
        commitment.clone(),
        BytesN::from_array(&env, &[0u8; 32]),
    ];

    let result = client.verify_proof(&user, &proof, &inputs);
    assert!(result, "valid EIP-197 proof should pass");
}
