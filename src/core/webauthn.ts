export class WebAuthnManager {
  /**
   * Check if WebAuthn is supported in the current environment
   */
  static isSupported(): boolean {
    return (
      typeof window !== 'undefined' &&
      typeof navigator !== 'undefined' &&
      typeof navigator.credentials !== 'undefined'
    );
  }

  /**
   * Register a new biometric credential
   */
  static async register(username: string, challenge: string): Promise<PublicKeyCredential> {
    if (!this.isSupported()) {
      throw new Error('WebAuthn is not supported in this environment');
    }

    const encoder = new TextEncoder();
    const challengeBuffer = encoder.encode(challenge);
    const userIdBuffer = encoder.encode(username);

    const options: CredentialCreationOptions = {
      publicKey: {
        challenge: challengeBuffer,
        rp: {
          name: 'ZK Auth Demo',
          id: window.location.hostname || 'localhost',
        },
        user: {
          id: userIdBuffer,
          name: username,
          displayName: username,
        },
        pubKeyCredParams: [
          { alg: -7, type: 'public-key' }, // ES256
          { alg: -257, type: 'public-key' }, // RS256
        ],
        authenticatorSelection: {
          userVerification: 'required',
        },
        timeout: 60000,
      },
    };

    const credential = await navigator.credentials.create(options);
    if (!credential) {
      throw new Error('Failed to create WebAuthn credential');
    }
    return credential as PublicKeyCredential;
  }

  /**
   * Authenticate with a registered biometric credential
   */
  static async authenticate(
    credentialIdHex: string,
    challenge: string,
  ): Promise<PublicKeyCredential> {
    if (!this.isSupported()) {
      throw new Error('WebAuthn is not supported in this environment');
    }

    const encoder = new TextEncoder();
    const challengeBuffer = encoder.encode(challenge);

    const matches = credentialIdHex.match(/.{1,2}/g);
    const credentialIdBuffer = new Uint8Array(
      matches ? matches.map((byte) => parseInt(byte, 16)) : [],
    );

    const options: CredentialRequestOptions = {
      publicKey: {
        challenge: challengeBuffer,
        allowCredentials: [
          {
            id: credentialIdBuffer,
            type: 'public-key',
          },
        ],
        userVerification: 'required',
        timeout: 60000,
      },
    };

    const assertion = await navigator.credentials.get(options);
    if (!assertion) {
      throw new Error('Failed to get WebAuthn assertion');
    }
    return assertion as PublicKeyCredential;
  }
}
