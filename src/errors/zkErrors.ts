export type ZkProofErrorCode =
  | 'ZK_INVALID_INPUT'
  | 'ZK_MISSING_ARTIFACTS'
  | 'ZK_CONSTRAINT_FAILED'
  | 'ZK_PROVE_FAILED'
  | 'ZK_UNKNOWN_ERROR';

export class ZkProofError extends Error {
  public readonly code: ZkProofErrorCode;
  public readonly details?: unknown;

  constructor(
    code: ZkProofErrorCode,
    message: string,
    details?: unknown,
    options?: { cause?: unknown },
  ) {
    super(message);
    this.name = 'ZkProofError';
    this.code = code;
    this.details = details;
    if (options && 'cause' in options) {
      (this as any).cause = options.cause;
    }
  }
}

function isConstraintRelatedMessage(message: string): boolean {
  const m = message.toLowerCase();

  // snarkjs / circuit runners commonly include one of these strings.
  return (
    m.includes('assert') ||
    m.includes('constraint') ||
    m.includes('witness') ||
    m.includes('unsatisfied') ||
    m.includes('satisfy') ||
    m.includes('invalid witness') ||
    m.includes('failed constraint')
  );
}

export function mapProverErrorToZkProofError(error: unknown): ZkProofError {
  if (error instanceof ZkProofError) return error;

  const message = error instanceof Error ? error.message : String(error);

  if (message.toLowerCase().includes('constraint') || isConstraintRelatedMessage(message)) {
    return new ZkProofError(
      'ZK_CONSTRAINT_FAILED',
      'Circuit constraint check failed. Please verify the provided inputs.',
      { originalMessage: message },
      { cause: error },
    );
  }

  if (message.toLowerCase().includes('missing') && message.toLowerCase().includes('wasm')) {
    return new ZkProofError(
      'ZK_MISSING_ARTIFACTS',
      'Missing circuit artifacts required to generate the proof.',
      { originalMessage: message },
      { cause: error },
    );
  }

  if (
    message.toLowerCase().includes('invalid proof input') ||
    message.toLowerCase().includes('invalid input')
  ) {
    return new ZkProofError(
      'ZK_INVALID_INPUT',
      'Invalid input provided for proof generation.',
      { originalMessage: message },
      { cause: error },
    );
  }

  return new ZkProofError(
    'ZK_PROVE_FAILED',
    'Failed to generate ZK proof. Please verify inputs and try again.',
    { originalMessage: message },
    { cause: error },
  );
}
