import { type ClientProtocolCircuitVerifier, type Tx } from '@aztec/circuit-types';

export class TestCircuitVerifier implements ClientProtocolCircuitVerifier {
  verifyProof(_tx: Tx): Promise<boolean> {
    return Promise.resolve(true);
  }
}
