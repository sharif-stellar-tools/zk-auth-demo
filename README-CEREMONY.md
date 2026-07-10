# Trusted Setup Ceremony — Auditor Guide

This document explains how to reproduce and verify the trusted setup for
`zk-auth-demo` circuits using the `ceremony` CLI.

## Prerequisites

- Node.js ≥ 18
- `npm install` (installs `snarkjs` and `commander`)
- Compiled circuit artifacts (`.r1cs`, `.wasm`) in `./ceremony-artifacts/`

## Commands

### `ceremony init` — Run the full trusted setup

```bash
npx ts-node scripts/ceremony/index.ts init \
  --power 12 \
  --entropy "your-random-entropy-string" \
  --output ./ceremony-artifacts \
  --circuits zk_auth
```

Options:
| Flag | Default | Description |
|---|---|---|
| `--power` | `12` | Powers of Tau exponent (2^12 = 4096 constraints) |
| `--entropy` | random beacon | Custom entropy for Phase 2 contribution |
| `--output` | `./ceremony-artifacts` | Directory for all ceremony artifacts |
| `--circuits` | `zk_auth` | Space-separated list of circuit names |

After running, a `ceremony-manifest.json` is written to the project root
containing the SHA-256 hash of every generated `.zkey` file.

### `ceremony verify <circuit-name>` — Verify published keys

```bash
npx ts-node scripts/ceremony/index.ts verify zk_auth \
  --artifacts ./ceremony-artifacts \
  --manifest ./ceremony-manifest.json
```

Exit codes:
- `0` — all checks passed; `.zkey` is unmodified
- `1` — hash mismatch or snarkjs verification failed; diff is printed

## What the verifier checks

1. **SHA-256 hash** — compares the `.zkey` on disk against the hash in
   `ceremony-manifest.json`. A mismatch means the file was tampered with.
2. **snarkjs zkey verify** — confirms the `.zkey` is cryptographically
   consistent with the Phase 1 Powers of Tau transcript.

## CI integration

Every PR that modifies circuit files automatically runs:

```bash
npx ts-node scripts/ceremony/index.ts verify zk_auth
```

See `.github/workflows/ceremony-verify.yml`.

## Security notes

- Never reuse entropy strings across ceremonies.
- The `ceremony-manifest.json` should be signed and published alongside
  the `.zkey` files so auditors can detect tampering.
- For production, use a multi-party ceremony with independent contributors.
