#![no_std]

//! On-chain Groth16 verifier for the zk-auth-demo circuit (BN254).
//!
//! Uses Stellar Protocol 25 native BN254 host functions:
//!   g1_msm, pairing_check
//!
//! Proof wire format (256 bytes, Ethereum-compatible uncompressed):
//!   pi_a  bytes [0..64]    — G1 point (x‖y, 32 bytes each)
//!   pi_b  bytes [64..192]  — G2 point (x1‖x0‖y1‖y0, 32 bytes each)
//!   pi_c  bytes [192..256] — G1 point (x‖y, 32 bytes each)
//!
//! Verification key is stored on-chain as a flat Vec<BytesN<32>> (chunks of
//! 32 bytes) so it survives contract upgrades without re-deploying the WASM.
//! See `VKey` layout in the comments below.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short,
    Address, Bytes, BytesN, Env, Symbol, Vec,
    crypto::bn254::{Bn254Fr, Bn254G1Affine, Bn254G2Affine},
};

// ─── Constants ────────────────────────────────────────────────────────────────

/// snarkjs groth16 proof: pi_a(64) + pi_b(128) + pi_c(64) = 256 bytes.
const PROOF_LEN: u32 = 256;

/// G1 uncompressed: 64 bytes.
const G1_LEN: u32 = 64;

/// G2 uncompressed: 128 bytes.
const G2_LEN: u32 = 128;

/// Chunks of 32 for vkey storage. G1 = 2 chunks, G2 = 4 chunks.
const G1_CHUNKS: u32 = 2;
const G2_CHUNKS: u32 = 4;

/// Vkey layout in the flat Vec<BytesN<32>> (indices of 32-byte chunks):
///
///   [0..2]   alpha_g1  (G1, 2 chunks)
///   [2..6]   beta_g2   (G2, 4 chunks)
///   [6..10]  gamma_g2  (G2, 4 chunks)
///   [10..14] delta_g2  (G2, 4 chunks)
///   [14..16] ic_0      (G1, 2 chunks)  — constant term
///   [16..18] ic_1      (G1, 2 chunks)  — commitment public input
///   [18..20] ic_2      (G1, 2 chunks)  — nonce public input
///
/// Total: 20 chunks = 640 bytes for a circuit with 2 public inputs.
#[allow(dead_code)]
const VKEY_CHUNKS_2_INPUTS: u32 = 20;

const CONFIG_KEY: Symbol = symbol_short!("CONFIG");
const VKEY_KEY: Symbol = symbol_short!("VKEY");

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 0,
    NotInitialized = 1,
    UninitializedConfig = 2,
    CommitmentExists = 3,
    CommitmentNotFound = 4,
    InvalidProof = 5,
}

// ─── Types ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct Config {
    /// Number of public inputs (currently 2: commitment + nonce).
    pub public_input_count: u32,
    /// Semver-ish version; bump if circuit changes.
    pub protocol_version: u32,
}

#[contracttype]
pub enum DataKey {
    Commitment(Address),
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct VerifierContract;

#[contractimpl]
impl VerifierContract {
    /// Deploy the verifier. Call once. `vkey_chunks` is the flat encoding of the
    /// verification key — see `VKEY_CHUNKS_2_INPUTS` layout above.
    pub fn __constructor(env: Env, config: Config, vkey_chunks: Vec<BytesN<32>>) {
        if env.storage().instance().has(&CONFIG_KEY) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }
        let expected = G1_CHUNKS + G2_CHUNKS * 3 + G1_CHUNKS * (1 + config.public_input_count);
        if vkey_chunks.len() != expected {
            panic_with_error!(&env, ContractError::UninitializedConfig);
        }
        env.storage().instance().set(&CONFIG_KEY, &config);
        env.storage().instance().set(&VKEY_KEY, &vkey_chunks);
    }

    /// Register a Poseidon commitment for a user. Must be called before verify_proof.
    pub fn register(env: Env, user: Address, commitment: BytesN<32>) {
        user.require_auth();
        if env
            .storage()
            .persistent()
            .has(&DataKey::Commitment(user.clone()))
        {
            panic_with_error!(&env, ContractError::CommitmentExists);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Commitment(user), &commitment);
    }

    /// Verify a Groth16 proof for a registered user.
    ///
    /// `proof_bytes` — 256-byte snarkjs groth16 proof (pi_a ‖ pi_b ‖ pi_c).
    /// `public_inputs` — [commitment, nonce], each a 32-byte BN254 scalar.
    ///
    /// Returns `true` for a valid proof, `false` for an invalid one.
    /// Never panics on bad proof data — only on misconfiguration.
    pub fn verify_proof(
        env: Env,
        user: Address,
        proof_bytes: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UninitializedConfig));

        let vkey: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&VKEY_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UninitializedConfig));

        // Structural checks — return false, not panic.
        if proof_bytes.len() != PROOF_LEN {
            return false;
        }
        if public_inputs.len() != config.public_input_count {
            return false;
        }

        // public_inputs[0] must match the user's registered commitment.
        let commitment: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::Commitment(user))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CommitmentNotFound));

        if public_inputs.get(0).unwrap() != commitment {
            return false;
        }

        verify_groth16(&env, &proof_bytes, &public_inputs, &vkey)
    }

    pub fn get_config(env: Env) -> Config {
        env.storage()
            .instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UninitializedConfig))
    }

    pub fn get_commitment(env: Env, user: Address) -> BytesN<32> {
        env.storage()
            .persistent()
            .get(&DataKey::Commitment(user))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CommitmentNotFound))
    }
}

// ─── Groth16 verification ─────────────────────────────────────────────────────

/// Run the Groth16 pairing equation using Soroban Protocol-25 BN254 host fns.
///
/// The equation is:
///   e(-pi_a, pi_b) · e(alpha, beta) · e(vk_x, gamma) · e(pi_c, delta) == 1
///
/// Which pairing_check verifies by checking that the product of all pairings == 1.
/// We negate pi_a on the G1 side (flip Y coordinate mod p) to avoid needing
/// a separate g1_neg call — the host's pairing_check accepts the check directly.
fn verify_groth16(
    env: &Env,
    proof: &Bytes,
    public_inputs: &Vec<BytesN<32>>,
    vkey: &Vec<BytesN<32>>,
) -> bool {
    let bn254 = env.crypto().bn254();

    // ── Deserialise proof ──
    let pi_a = g1_from_proof(env, proof, 0);
    let pi_b = g2_from_proof(env, proof, G1_LEN);
    let pi_c = g1_from_proof(env, proof, G1_LEN + G2_LEN);

    // ── Deserialise verification key ──
    // Vkey chunk layout (each chunk 32 bytes):
    //   [0..2]      alpha_g1
    //   [2..6]      beta_g2
    //   [6..10]     gamma_g2
    //   [10..14]    delta_g2
    //   [14..16]    IC[0]
    //   [16..18]    IC[1]   (one per public input)
    //   [18..20]    IC[2]
    let alpha = g1_from_vkey(env, vkey, 0);
    let beta  = g2_from_vkey(env, vkey, G1_CHUNKS);
    let gamma = g2_from_vkey(env, vkey, G1_CHUNKS + G2_CHUNKS);
    let delta = g2_from_vkey(env, vkey, G1_CHUNKS + G2_CHUNKS * 2);

    // ── Compute vk_x = IC[0] + sum(public_inputs[i] * IC[i+1]) ──
    let ic_base_chunk = G1_CHUNKS + G2_CHUNKS * 3;
    let ic_0 = g1_from_vkey(env, vkey, ic_base_chunk);

    // Accumulate: vk_x starts at IC[0], then add scalar*IC[i+1] for each input.
    let mut ic_points: Vec<Bn254G1Affine> = Vec::new(env);
    let mut ic_scalars: Vec<Bn254Fr> = Vec::new(env);

    // IC[0] with scalar = 1
    ic_points.push_back(ic_0);
    ic_scalars.push_back(scalar_one(env));

    for i in 0..public_inputs.len() {
        let ic_i = g1_from_vkey(env, vkey, ic_base_chunk + G1_CHUNKS * (i + 1));
        let s = fr_from_bytes(env, &public_inputs.get(i).unwrap());
        ic_points.push_back(ic_i);
        ic_scalars.push_back(s);
    }

    let vk_x = bn254.g1_msm(ic_points, ic_scalars);

    // ── Negate pi_a (flip G1 Y coordinate mod p) ──
    let neg_pi_a = g1_negate(env, pi_a);

    // ── Pairing check: e(-pi_a,pi_b) · e(alpha,beta) · e(vk_x,gamma) · e(pi_c,delta) == 1 ──
    let mut g1s: Vec<Bn254G1Affine> = Vec::new(env);
    let mut g2s: Vec<Bn254G2Affine> = Vec::new(env);

    g1s.push_back(neg_pi_a);   g2s.push_back(pi_b);
    g1s.push_back(alpha);      g2s.push_back(beta);
    g1s.push_back(vk_x);       g2s.push_back(gamma);
    g1s.push_back(pi_c);       g2s.push_back(delta);

    bn254.pairing_check(g1s, g2s)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extract a G1 point (64 bytes) from the proof at byte offset `offset`.
fn g1_from_proof(env: &Env, proof: &Bytes, offset: u32) -> Bn254G1Affine {
    let mut arr = [0u8; 64];
    for i in 0..64u32 {
        arr[i as usize] = proof.get(offset + i).unwrap();
    }
    Bn254G1Affine::from_bytes(BytesN::from_array(env, &arr))
}

/// Extract a G2 point (128 bytes) from the proof at byte offset `offset`.
fn g2_from_proof(env: &Env, proof: &Bytes, offset: u32) -> Bn254G2Affine {
    let mut arr = [0u8; 128];
    for i in 0..128u32 {
        arr[i as usize] = proof.get(offset + i).unwrap();
    }
    Bn254G2Affine::from_bytes(BytesN::from_array(env, &arr))
}

/// Read a G1 point from the flat vkey chunk array starting at chunk index `start`.
fn g1_from_vkey(env: &Env, vkey: &Vec<BytesN<32>>, start: u32) -> Bn254G1Affine {
    let mut arr = [0u8; 64];
    for chunk in 0..2u32 {
        let bytes: BytesN<32> = vkey.get(start + chunk).unwrap();
        for b in 0..32u32 {
            arr[(chunk * 32 + b) as usize] = bytes.get(b).unwrap();
        }
    }
    Bn254G1Affine::from_bytes(BytesN::from_array(env, &arr))
}

/// Read a G2 point from the flat vkey chunk array starting at chunk index `start`.
fn g2_from_vkey(env: &Env, vkey: &Vec<BytesN<32>>, start: u32) -> Bn254G2Affine {
    let mut arr = [0u8; 128];
    for chunk in 0..4u32 {
        let bytes: BytesN<32> = vkey.get(start + chunk).unwrap();
        for b in 0..32u32 {
            arr[(chunk * 32 + b) as usize] = bytes.get(b).unwrap();
        }
    }
    Bn254G2Affine::from_bytes(BytesN::from_array(env, &arr))
}

/// Convert a 32-byte big-endian scalar to Bn254Fr.
fn fr_from_bytes(_env: &Env, bytes: &BytesN<32>) -> Bn254Fr {
    Bn254Fr::from_bytes(bytes.clone())
}

/// Return the scalar field element 1.
fn scalar_one(env: &Env) -> Bn254Fr {
    use soroban_sdk::U256;
    Bn254Fr::from(U256::from_u32(env, 1))
}

/// Negate a G1 point. The SDK implements `Neg for &Bn254G1Affine` natively.
fn g1_negate(_env: &Env, p: Bn254G1Affine) -> Bn254G1Affine {
    -p
}

mod test;
