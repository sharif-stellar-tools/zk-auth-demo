import {
  runCircuitBenchmark,
  compareHardwareRuntimes,
  formatBenchmarkDashboard,
  KNOWN_CIRCUITS,
} from '../src/benchmark';

describe('Circuit Benchmarking Suite', () => {
  it('should have known circuits configured with valid constraints', () => {
    expect(KNOWN_CIRCUITS.nullifier.constraintCount).toBe(480);
    expect(KNOWN_CIRCUITS.merkle.constraintCount).toBe(12480);
    expect(KNOWN_CIRCUITS.fullAuth.constraintCount).toBe(13520);
  });

  it('should benchmark circuit on CPU runtime', async () => {
    const metrics = await runCircuitBenchmark({
      circuitName: 'nullifier',
      target: 'CPU',
      iterations: 2,
    });

    expect(metrics.circuitName).toBe('NullifierHasher');
    expect(metrics.target).toBe('CPU');
    expect(metrics.constraintCount).toBe(480);
    expect(metrics.iterations).toBe(2);
    expect(metrics.avgWitnessTimeMs).toBeGreaterThan(0);
    expect(metrics.avgProofTimeMs).toBeGreaterThan(0);
    expect(metrics.totalTimeMs).toBeGreaterThan(0);
    expect(metrics.cpuModel.length).toBeGreaterThan(0);
    expect(metrics.osPlatform.length).toBeGreaterThan(0);
  });

  it('should benchmark circuit on WebAssembly runtime', async () => {
    const metrics = await runCircuitBenchmark({
      circuitName: 'merkle',
      target: 'WebAssembly',
      iterations: 2,
    });

    expect(metrics.circuitName).toBe('MerkleProofVerifier');
    expect(metrics.target).toBe('WebAssembly');
    expect(metrics.constraintCount).toBe(12480);
    expect(metrics.avgWitnessTimeMs).toBeGreaterThan(0);
    expect(metrics.avgProofTimeMs).toBeGreaterThan(0);
  });

  it('should compare hardware runtimes and determine speedup', async () => {
    const comparison = await compareHardwareRuntimes('nullifier', 2);

    expect(comparison.circuitName).toBe('NullifierHasher');
    expect(comparison.constraintCount).toBe(480);
    expect(comparison.cpuMetrics.target).toBe('CPU');
    expect(comparison.wasmMetrics.target).toBe('WebAssembly');
    expect(comparison.speedupRatio).toBeGreaterThan(0);
    expect(['CPU', 'WebAssembly']).toContain(comparison.fasterTarget);
  });

  it('should format a visual dashboard report for multiple circuits', async () => {
    const repNullifier = await compareHardwareRuntimes('nullifier', 1);
    const repMerkle = await compareHardwareRuntimes('merkle', 1);

    const dashboard = formatBenchmarkDashboard([repNullifier, repMerkle]);

    expect(dashboard).toContain('ZK-AUTH CIRCUIT BENCHMARKING DASHBOARD');
    expect(dashboard).toContain('NullifierHasher');
    expect(dashboard).toContain('MerkleProofVerifier');
    expect(dashboard).toContain('Speedup');
  });
});
