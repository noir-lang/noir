import { type AuthWitnessProvider } from '@aztec/aztec.js/account';
import { AuthWitness, type CompleteAddress, type GrumpkinScalar } from '@aztec/circuit-types';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type Fr } from '@aztec/foundation/fields';

import { DefaultAccountContract } from '../defaults/account_contract.js';
import { SchnorrSingleKeyAccountContractArtifact } from './artifact.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures verified against
 * the note encryption key, relying on a single private key for both encryption and authentication.
 */
export class SingleKeyAccountContract extends DefaultAccountContract {
  constructor(private encryptionPrivateKey: GrumpkinScalar) {
    super(SchnorrSingleKeyAccountContractArtifact as ContractArtifact);
  }

  getDeploymentArgs(): undefined {
    return undefined;
  }

  getAuthWitnessProvider(account: CompleteAddress): AuthWitnessProvider {
    return new SingleKeyAuthWitnessProvider(this.encryptionPrivateKey, account);
  }
}

/**
 * Creates auth witnesses using Schnorr signatures and including the partial address and public key
 * in the witness, so verifiers do not need to store the public key and can instead validate it
 * by reconstructing the current address.
 */
class SingleKeyAuthWitnessProvider implements AuthWitnessProvider {
  constructor(private privateKey: GrumpkinScalar, private account: CompleteAddress) {}

  createAuthWit(messageHash: Fr): Promise<AuthWitness> {
    const schnorr = new Schnorr();
    const signature = schnorr.constructSignature(messageHash.toBuffer(), this.privateKey);
    const witness = [...this.account.publicKeys.toFields(), ...signature.toBuffer(), this.account.partialAddress];
    return Promise.resolve(new AuthWitness(messageHash, witness));
  }
}
