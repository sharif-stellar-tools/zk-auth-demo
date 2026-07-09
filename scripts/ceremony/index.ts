#!/usr/bin/env node
// SPDX-License-Identifier: MIT
/**
 * ceremony - Trusted Setup CLI for zk-auth-demo
 *
 * Automates Phase 1 (Powers of Tau) and Phase 2 (circuit-specific) trusted
 * setup steps, and provides a `verify` command for third-party auditors.
 *
 * Usage:
 *   npx ts-node scripts/ceremony/index.ts init [options]
 *   npx ts-node scripts/ceremony/index.ts verify <circuit-name>
 *
 * See README-CEREMONY.md for a full step-by-step auditor guide.
 */

import { program } from 'commander';
import { runInit } from './commands/init';
import { runVerify } from './commands/verify';

program
  .name('ceremony')
  .description('Trusted Setup CLI — Phase 1 (Powers of Tau) and Phase 2 (circuit-specific)')
  .version('1.0.0');

program
  .command('init')
  .description('Run Powers of Tau (Phase 1) and circuit-specific setup (Phase 2)')
  .option('-p, --power <number>', 'Powers of Tau exponent (circuit constraint count = 2^power)', '12')
  .option('-e, --entropy <string>', 'Custom entropy string for randomness beacon (optional)')
  .option('-o, --output <dir>', 'Output directory for ceremony artifacts', './ceremony-artifacts')
  .option('-c, --circuits <names...>', 'Circuit names to set up', ['zk_auth'])
  .action(async (opts) => {
    await runInit({
      power: parseInt(opts.power, 10),
      entropy: opts.entropy,
      outputDir: opts.output,
      circuits: opts.circuits,
    });
  });

program
  .command('verify <circuit-name>')
  .description('Verify a .zkey file against the published Phase 1 transcript and manifest')
  .option('-a, --artifacts <dir>', 'Ceremony artifacts directory', './ceremony-artifacts')
  .option('-m, --manifest <file>', 'Path to ceremony-manifest.json', './ceremony-manifest.json')
  .action(async (circuitName: string, opts) => {
    const exitCode = await runVerify({
      circuitName,
      artifactsDir: opts.artifacts,
      manifestPath: opts.manifest,
    });
    process.exit(exitCode);
  });

program.parseAsync(process.argv).catch((err: unknown) => {
  console.error('ceremony error:', err instanceof Error ? err.message : err);
  process.exit(1);
});
