import { AztecAddress, Signature, TxRequest } from '../circuits.js';

export interface KeyStore {
  addAccount(): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  getSigningPublicKeys(): Promise<AztecAddress[]>;
  signTxRequest(txRequest: TxRequest): Promise<Signature>;
}
