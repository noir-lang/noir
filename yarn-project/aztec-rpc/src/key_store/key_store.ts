import { AztecAddress, TxRequest } from '@aztec/circuits.js';
import { Signature } from '../circuits.js';

export interface KeyStore {
  addAccount(): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  getAccountPrivateKey(address: AztecAddress): Promise<Buffer>;
  getSigningPublicKeys(): Promise<AztecAddress[]>;
  signTxRequest(txRequest: TxRequest): Promise<Signature>;
}
