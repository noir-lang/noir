import { type ClientProtocolCircuitVerifier, type Tx } from '@aztec/circuit-types';
import { type VerificationKeys, getMockVerificationKeys } from '@aztec/circuits.js';

export class TestCircuitVerifier implements ClientProtocolCircuitVerifier {
  verifyProof(_tx: Tx): Promise<boolean> {
    return Promise.resolve(true);
  }

  getVerificationKeys(): Promise<VerificationKeys> {
    return Promise.resolve(getMockVerificationKeys());
  }
}
