import { AztecAddress, TxRequest } from '@aztec/circuits.js';
import { Signature } from '../circuits.js';

export interface KeyStore {
  addAccount(): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  getSigningPublicKeys(): Promise<AztecAddress[]>;
  signTxRequest(txRequest: TxRequest): Promise<Signature>;
}
