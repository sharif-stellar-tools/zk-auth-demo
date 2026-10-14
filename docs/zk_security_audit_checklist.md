# Soroban Zero-Knowledge Security Audit Checklist & Best Practices

This guide establishes a comprehensive security auditing checklist and mitigation reference for Zero-Knowledge (ZK) applications, Circom/Groth16 verifiers, and privacy protocols deployed on Stellar Soroban.

---

## 🎯 Executive Audit Categories

1. **Circuit Integrity & Constraint Completeness**
2. **Public Signals & Nullifier Malleability**
3. **Soroban Contract Storage & State Collision**
4. **On-Chain Verifier & Key Rotation Security**
5. **Front-Running & Transaction Replay Defenses**

---

## 1. 🔍 Circuit Integrity & Constraint Completeness

### Under-Constrained Signals
- [ ] **Ensure All Signals are Constrained**: Verify that every intermediate signal `s` is constrained via quadratic R1CS equations (`a * b === c`). Unconstrained signals allow adversaries to compute valid proofs with arbitrary values.
- [ ] **Boolean & Range Constraints**: Any signal representing a boolean flag must enforce `b * (1 - b) === 0`. Numeric values representing amounts or timestamps must be explicitly bounded to prevent field overflow wrap-around modulo $p$ (e.g., $p = 21888242871839275222246405745257275088548364400416034343698204186575808495617$ for BN254).
- [ ] **Division by Zero Protection**: In Circom circuits, ensure denominator checks exist before division operations (`inv * denom === 1`).

---

## 2. 🛡️ Public Signals & Nullifier Malleability

### Public Input Binding
- [ ] **Bind Proof to Execution Context**: Proof verification alone does NOT guarantee that the proof was intended for the caller or the current transaction. Public signals MUST include:
  1. `recipient`: Prevents an adversary from intercepting an off-chain proof and submitting it with their own address.
  2. `amount`: Prevents altering the withdrawal or payout value.
  3. `contract_id`: Prevents cross-contract proof replay on other deployed instances.
- [ ] **Nullifier Determinism & Unique Insertion**:
  - Nullifiers must be derived as a one-way collision-resistant hash (e.g. `Poseidon(secret, pathIndex)`).
  - The Soroban contract must record spent nullifiers in persistent storage and reject duplicates with `Error::NullifierAlreadySpent`.

---

## 3. 🔐 Soroban Contract Storage & Key Collision

### Storage Hygiene
- [ ] **Distinct Storage Keys**: Prevent collisions between protocol parameters, admin roles, and nullifier sets:
  ```rust
  #[contracttype]
  pub enum DataKey {
      Admin,
      VerifierKey,
      Nullifier(BytesN<32>),
      TotalSettled,
  }
  ```
- [ ] **TTL & Instance vs Persistent Storage**:
  - Nullifier states must ALWAYS be stored in **Persistent Storage** with automated `extend_ttl` bumps. Never store spent nullifiers in Instance storage where eviction would re-enable double-spending.

---

## 4. 🔑 On-Chain Verifier & Key Rotation

### Verification Key Validation
- [ ] **Groth16 Pairing Completeness**: Ensure the on-chain Soroban verifier checks all pairing elements $(\alpha, \beta, \gamma, \delta)$ and correctly maps public inputs $L_{i}$ into the $G_1$ multi-scalar multiplication.
- [ ] **Two-Step Admin Key Rotation**: If the verifier key can be upgraded (e.g. for circuit updates), enforce a two-step timelock or multisig authorization to prevent rogue admin takeovers.

---

## 5. ⚡ Front-Running & Replay Defenses

### Transaction Envelope Guards
- [ ] **Sub-Invocation Authorization**: Always wrap Soroban verification in `require_auth()` for the caller to prevent mempool front-running.
- [ ] **Proof Malleability Check**: Groth16 proofs admit $(A, B, C) \sim (-A, -B, C)$. Enforce strict canonical representation of curve coordinates in point deserialization to eliminate proof malleability.
