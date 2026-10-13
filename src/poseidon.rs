// Native Poseidon Hash Implementation for Soroban Smart Contracts
// Finite field sponge construction with t=3 (2 inputs, 1 capacity) over Goldilocks prime p = 2^64 - 2^32 + 1

pub const FIELD_MODULUS: u128 = 18446744069414584321; // 2^64 - 2^32 + 1

// Round constants for full rounds
const ROUND_CONSTANTS: [u64; 18] = [
    0x2c6b41bf63673c68, 0x1f92e46b9a842718, 0x7b9c240974b130e5,
    0x4036f06422d3d9e0, 0x5b38a7c290a3c26b, 0x6e2b904971c69a03,
    0x3f5c8710bc8201a4, 0x762810a9c824b219, 0x18c028a379f821bc,
    0x54a0128479c0281b, 0x489b0281749b0281, 0x2948b02175940291,
    0x6194b029184b0291, 0x30948b029184b029, 0x510948b029184b02,
    0x240948b029184b02, 0x720948b029184b02, 0x190948b029184b02,
];

// 3x3 MDS Matrix for maximal diffusion
const MDS_MATRIX: [[u64; 3]; 3] = [
    [2, 1, 1],
    [1, 2, 1],
    [1, 1, 2],
];

#[inline(always)]
fn field_add(a: u64, b: u64) -> u64 {
    let sum = (a as u128) + (b as u128);
    (sum % FIELD_MODULUS) as u64
}

#[inline(always)]
fn field_mul(a: u64, b: u64) -> u64 {
    let prod = (a as u128) * (b as u128);
    (prod % FIELD_MODULUS) as u64
}

#[inline(always)]
fn sbox(x: u64) -> u64 {
    // x^5 mod p
    let x2 = field_mul(x, x);
    let x4 = field_mul(x2, x2);
    field_mul(x4, x)
}

/// Computes native Poseidon hash of two 64-bit field elements (t=3 sponge)
pub fn poseidon_hash_2(left: u64, right: u64) -> u64 {
    let mut state = [
        (left as u128 % FIELD_MODULUS) as u64,
        (right as u128 % FIELD_MODULUS) as u64,
        0u64, // capacity element
    ];

    let rounds = 6;
    for r in 0..rounds {
        // 1. Add round constants
        state[0] = field_add(state[0], ROUND_CONSTANTS[r * 3]);
        state[1] = field_add(state[1], ROUND_CONSTANTS[r * 3 + 1]);
        state[2] = field_add(state[2], ROUND_CONSTANTS[r * 3 + 2]);

        // 2. SubWords (S-box)
        state[0] = sbox(state[0]);
        state[1] = sbox(state[1]);
        state[2] = sbox(state[2]);

        // 3. MixLayer (MDS matrix multiplication)
        let s0 = state[0];
        let s1 = state[1];
        let s2 = state[2];

        state[0] = field_add(
            field_add(field_mul(s0, MDS_MATRIX[0][0]), field_mul(s1, MDS_MATRIX[0][1])),
            field_mul(s2, MDS_MATRIX[0][2]),
        );
        state[1] = field_add(
            field_add(field_mul(s0, MDS_MATRIX[1][0]), field_mul(s1, MDS_MATRIX[1][1])),
            field_mul(s2, MDS_MATRIX[1][2]),
        );
        state[2] = field_add(
            field_add(field_mul(s0, MDS_MATRIX[2][0]), field_mul(s1, MDS_MATRIX[2][1])),
            field_mul(s2, MDS_MATRIX[2][2]),
        );
    }

    state[0]
}
