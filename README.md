<div align="center">
  <h1>zk-auth-demo</h1>
  <p><strong>Zero-knowledge proof verification circuits for decentralized identity.</strong></p>
</div>

<br />

## 📖 Overview

zk-auth-demo is a critical component of our decentralized ecosystem. This repository contains the source code, tests, and deployment configurations necessary to run the service. Built with modern, enterprise-grade architecture, it ensures high availability, secure execution, and seamless integration with the broader network.

## 🧠 How It Works

Authentication here is built on a single idea from zero-knowledge proofs: **you can prove you know a secret without ever revealing it.** Instead of sending a password to a server that checks it, you send a cryptographic proof that you know the password. The server learns only one bit of information: "yes, this person knows the secret" or "no, they don't."

### The everyday analogy

Imagine a locked door with a number pad. Normally you would type your PIN, and whoever is watching the keypad now knows your PIN. A zero-knowledge proof is like proving you can open the door without anyone seeing which buttons you press. You demonstrate the _result_ (the door opens) without exposing the _secret_ (the PIN).

### Public vs. private inputs

A zero-knowledge circuit is just a set of mathematical rules (constraints) that the prover must satisfy. Its inputs come in two flavors:

| Input | Visibility | What it is | Who sees it |
| --- | --- | --- | --- |
| `secret` | **Private** (the "witness") | The value only the user knows, for example a password or private key. It is fed into the proof but never leaves the user's device. | The user only |
| `commitment` | **Public** | A one-way fingerprint of the secret, `commitment = Poseidon(secret)`, published once at registration. | Everyone (server, verifier) |
| `nonce` / `challenge` | **Public** | A fresh random value issued per login so an old proof cannot be replayed. | Everyone |

The private input is the thing being protected. The public inputs are the facts everyone agrees on before the proof is checked. A valid proof convinces the verifier that the private `secret` matches the public `commitment`, without the verifier ever seeing `secret`.

In code, this is the boundary you see in [`src/prover.ts`](./src/prover.ts): `generateProof(secret)` takes the private input and returns a proof object. The secret stays in; only the proof comes out.

### The core constraint

The circuit enforces one essential equation:

```
Poseidon(secret) == commitment
```

That is the whole authentication check, expressed as a constraint the proof must satisfy. To bind the proof to a single login attempt and stop replay attacks, a second constraint ties in the public `nonce`:

```
1. h = Poseidon(secret)      // recompute the fingerprint inside the circuit
2. assert h == commitment    // it must match the public commitment
3. expose nonce as public    // the proof is valid for this challenge only
```

If and only if the prover supplies a `secret` whose Poseidon hash equals the published `commitment`, the constraints are satisfied and the proof verifies. Because the proof reveals nothing about `secret` beyond the truth of that equation, the password is never transmitted or stored.

### Why Poseidon for the hash

The fingerprint function inside the circuit is **Poseidon**, a hash function designed specifically for zero-knowledge systems.

Everyday hash functions like SHA-256 are fast on normal CPUs but very expensive to express as circuit constraints, because they shuffle data bit by bit. ZK circuits work over large prime-field numbers, not bits, so a SHA-256 hash can cost tens of thousands of constraints. More constraints means slower proofs and bigger keys.

Poseidon is "ZK-friendly": it is built from simple arithmetic (additions and raising numbers to a small power) over the same prime field the circuit already uses. The same hash needs far fewer constraints, so proofs are smaller and faster to generate and verify, while keeping strong collision resistance. That trade is why ZK-authentication and Merkle-tree designs almost always reach for Poseidon instead of SHA-256.

### Putting it together

```
   secret (private)            commitment (public)   nonce (public)
        │                              │                  │
        ▼                              │                  │
  ┌───────────────┐                    │                  │
  │  ZK Circuit   │  h = Poseidon(secret)                 │
  │  (snarkjs)    │  assert h == commitment ◄─────────────┤
  └──────┬────────┘                                       │
         │ proof (reveals nothing about secret)           │
         ▼                                                ▼
                 Verifier checks proof + public inputs
                 → accepts login if and only if it is valid
```

Proof generation and verification run on the [snarkjs](https://github.com/iden3/snarkjs) proving stack. The takeaway for a developer integrating this: you only ever handle the user's secret locally to build a proof, and you only ever publish the public `commitment` and per-login `nonce`. The secret itself never touches the wire.

## 🎯 React Component: ZKLoginButton

We provide a reusable React component for integrating ZK authentication into your application.

### Installation

```bash
npm install zk-auth-demo
```

### Basic Usage

```typescript
import React, { useState } from 'react';
import { ZKLoginButton, ZkProver } from 'zk-auth-demo';

export const LoginPage = () => {
  const prover = new ZkProver();
  const [proof, setProof] = useState<string | null>(null);

  // Secret stays on user's device — never transmitted
  const userSecret = 'keep-this-safe';

  const handleGenerateProof = async () => {
    return await prover.generateProof(userSecret);
  };

  const handleSuccess = (proof: string) => {
    setProof(proof);
    // Send proof to backend for verification
    console.log('Proof ready, sending to server...');
  };

  const handleError = (error: Error) => {
    console.error('Authentication failed:', error.message);
  };

  return (
    <ZKLoginButton
      onGenerateProof={handleGenerateProof}
      onSuccess={handleSuccess}
      onError={handleError}
      label="Login with ZK"
    />
  );
};
```

### Component Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `onGenerateProof` | `() => Promise<string>` | Yes | Async callback that generates the ZK proof. Secret handling stays external. |
| `onSuccess` | `(proof: string) => void` | Yes | Called when proof is successfully generated. |
| `onError` | `(error: Error) => void` | No | Called if proof generation fails. |
| `label` | `string` | No | Button text (default: "Login with ZK") |
| `className` | `string` | No | CSS class for custom styling |

### Security Notes

- **Secrets never leave the user's device** — pass secret handling via `onGenerateProof` callback
- **Errors are sanitized** — no internal details exposed to users
- **Loading state prevents race conditions** — button disabled during proof generation
- **Proof validation** — empty or invalid proofs are rejected before callback

### Accessibility

- Button includes `aria-busy` and `aria-label` attributes
- Error messages use `role="alert"` and `aria-live="polite"` for screen readers
- Keyboard accessible — fully operable with Enter/Space

## ✨ Key Features

- **Robust Architecture**: Designed to handle high-throughput and scale horizontally.
- **Secure by Default**: Follows industry-standard security practices and comprehensive auditing guidelines.
- **Extensible Integration**: Exposes clean, well-documented interfaces for third-party extensions.
- **Comprehensive Testing**: Backed by a strict CI/CD pipeline enforcing an 85%+ code coverage requirement.

## 🚀 Getting Started

### Prerequisites
- Make sure you have the latest stable versions of our core toolchains (e.g., Node.js, Rust/Cargo) installed.
- Ensure Docker is installed for running localized integration environments.

### Local Installation

\\\ash
# Clone the repository
git clone https://github.com/YourOrganization/zk-auth-demo.git
cd zk-auth-demo

# Install dependencies and build
# (Refer to package.json or Cargo.toml for specific build commands)
\\\

## 🤝 Contributing
We welcome contributions from the community! Please read our [Contributing Guidelines](./CONTRIBUTING.md) to get started. Before submitting a Pull Request, ensure that you have reviewed our [Code of Conduct](./CODE_OF_CONDUCT.md).

## 📄 License
This project is licensed under the MIT License. See the LICENSE file for more details.
