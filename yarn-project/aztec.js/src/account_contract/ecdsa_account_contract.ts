import { Ecdsa } from '@aztec/circuits.js/barretenberg';
import { ContractArtifact } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { AuthWitness, CompleteAddress } from '@aztec/types';

import { AuthWitnessProvider } from '../account/interface.js';
import EcdsaAccountContractArtifact from './artifacts/EcdsaAccount.json' assert { type: 'json' };
import { BaseAccountContract } from './base_account_contract.js';

/**
 * Account contract that authenticates transactions using ECDSA signatures
 * verified against a secp256k1 public key stored in an immutable encrypted note.
 */
export class EcdsaAccountContract extends BaseAccountContract {
  constructor(private signingPrivateKey: Buffer) {
    super(EcdsaAccountContractArtifact as ContractArtifact);
  }

  getDeploymentArgs() {
    const signingPublicKey = new Ecdsa().computePublicKey(this.signingPrivateKey);
    return [signingPublicKey.subarray(0, 32), signingPublicKey.subarray(32, 64)];
  }

  getAuthWitnessProvider(_address: CompleteAddress): AuthWitnessProvider {
    return new EcdsaAuthWitnessProvider(this.signingPrivateKey);
  }
}

/** Creates auth witnesses using ECDSA signatures. */
class EcdsaAuthWitnessProvider implements AuthWitnessProvider {
  constructor(private signingPrivateKey: Buffer) {}

  createAuthWitness(message: Fr): Promise<AuthWitness> {
    const ecdsa = new Ecdsa();
    const signature = ecdsa.constructSignature(message.toBuffer(), this.signingPrivateKey);
    return Promise.resolve(new AuthWitness(message, [...signature.r, ...signature.s]));
  }
}
