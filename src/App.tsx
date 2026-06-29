import React, { useMemo, useState } from 'react';
import { ZkProver } from './prover';
import { ZkProofError } from './errors/zkErrors';
import { ZKLoginButton } from './components/ZKLoginButton';

type SubmitResponse = {
  ok: boolean;
  message?: string;
  details?: unknown;
};

export default function App() {
  const prover = useMemo(() => new ZkProver(), []);

  // User secret (private witness input). Must never be logged.
  const [secret, setSecret] = useState('');

  // snarkjs requires paths to wasm + zkey artifacts.
  const [wasmPath, setWasmPath] = useState('/circuits/circuit.wasm');
  const [zkeyPath, setZkeyPath] = useState('/circuits/circuit_final.zkey');

  // Backend verification endpoint.
  const [verifyUrl, setVerifyUrl] = useState('/api/verify');

  const [proofJson, setProofJson] = useState<string>('');
  const [publicSignalsJson, setPublicSignalsJson] = useState<string>('');
  const [submitResult, setSubmitResult] = useState<SubmitResponse | null>(null);

  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleGenerateProof = async () => {
    // Input validation (basic)
    if (!secret || secret.trim().length === 0) {
      throw new ZkProofError('ZK_INVALID_INPUT', 'Secret is required.', {
        secretLength: secret.length,
      });
    }

    // TODO: Map your secret into the circuit's expected witness format.
    // Here we assume the circuit has an input named `secret`.
    const witnessInput = {
      secret: secret.trim(),
    };

    const { proof, publicSignals } = await prover.generateProof(witnessInput, wasmPath, zkeyPath);

    // ZKLoginButton expects a string proof to validate.
    const proofStr = JSON.stringify(proof);

    setProofJson(proofStr);
    setPublicSignalsJson(JSON.stringify(publicSignals));

    return proofStr;
  };

  const handleSubmit = async () => {
    if (!proofJson || !publicSignalsJson) return;

    setIsSubmitting(true);
    setSubmitResult(null);

    try {
      const res = await fetch(verifyUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          // Keep payload explicit so the backend can parse it.
          proof: JSON.parse(proofJson),
          publicSignals: JSON.parse(publicSignalsJson),
        }),
      });

      const data = (await res.json()) as SubmitResponse;
      setSubmitResult(data);
    } catch (e) {
      setSubmitResult({
        ok: false,
        message: e instanceof Error ? e.message : 'Submit failed',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  const canSubmit = Boolean(proofJson && publicSignalsJson) && !isSubmitting;

  return (
    <div className="page">
      <h1>ZK Proof Generator (Browser)</h1>
      <p className="subtitle">
        Enter your secret, generate a zero-knowledge proof in the browser, then submit it for
        verification.
      </p>

      <div className="card">
        <label className="label">
          Secret (private witness)
          <textarea
            value={secret}
            onChange={(e) => setSecret(e.target.value)}
            placeholder="Type your secret"
            rows={4}
            className="textarea"
          />
        </label>

        <div className="grid">
          <label className="label">
            wasmPath
            <input
              value={wasmPath}
              onChange={(e) => setWasmPath(e.target.value)}
              className="input"
            />
          </label>

          <label className="label">
            zkeyPath
            <input
              value={zkeyPath}
              onChange={(e) => setZkeyPath(e.target.value)}
              className="input"
            />
          </label>
        </div>

        <div className="grid">
          <label className="label">
            verifyUrl (POST)
            <input
              value={verifyUrl}
              onChange={(e) => setVerifyUrl(e.target.value)}
              className="input"
            />
          </label>

          <div className="label">
            <div className="labelText">Actions</div>
            <div className="actions">
              <ZKLoginButton
                label="Generate proof"
                onGenerateProof={handleGenerateProof}
                onSuccess={() => {
                  // proofJson/publicSignalsJson were set inside generate
                }}
                onError={(err) => {
                  setSubmitResult({ ok: false, message: err.message });
                }}
              />

              <button
                className="button primary"
                onClick={handleSubmit}
                disabled={!canSubmit}
                aria-busy={isSubmitting}
              >
                {isSubmitting ? 'Submitting...' : 'Submit for verification'}
              </button>
            </div>
          </div>
        </div>

        {submitResult && (
          <div className={`result ${submitResult.ok ? 'ok' : 'err'}`} role="status">
            <pre>{JSON.stringify(submitResult, null, 2)}</pre>
          </div>
        )}
      </div>

      <div className="card split">
        <div>
          <div className="sectionTitle">Proof</div>
          <textarea
            className="output"
            value={proofJson}
            readOnly
            placeholder="Generated proof will appear here"
            rows={10}
          />
        </div>

        <div>
          <div className="sectionTitle">Public signals</div>
          <textarea
            className="output"
            value={publicSignalsJson}
            readOnly
            placeholder="Generated public signals will appear here"
            rows={10}
          />
        </div>
      </div>
    </div>
  );
}
