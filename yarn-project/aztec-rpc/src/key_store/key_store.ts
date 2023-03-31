import { AztecAddress, TxRequest } from '@aztec/circuits.js';
import { Point } from '@aztec/foundation';
import { Signature } from '../circuits.js';

export interface KeyStore {
  addAccount(): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  getAccountPrivateKey(address: AztecAddress): Promise<Buffer>;
  getSigningPublicKeys(): Promise<Point[]>;
  signTxRequest(txRequest: TxRequest): Promise<Signature>;
}
