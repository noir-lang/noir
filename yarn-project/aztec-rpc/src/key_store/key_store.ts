import { AztecAddress } from '@aztec/foundation';
import { EcdsaSignature, TxRequest } from '@aztec/circuits.js';
import { Point } from '@aztec/foundation';

export interface KeyStore {
  addAccount(): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  getAccountPrivateKey(address: AztecAddress): Promise<Buffer>;
  getAccountPublicKey(address: AztecAddress): Promise<Point>;
  getSigningPublicKeys(): Promise<Point[]>;
  signTxRequest(txRequest: TxRequest): Promise<EcdsaSignature>;
}
