import { type AuthWitnessProvider } from '@aztec/aztec.js/account';
import { AuthWitness, type CompleteAddress, type GrumpkinScalar } from '@aztec/circuit-types';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type Fr } from '@aztec/foundation/fields';

import { DefaultAccountContract } from '../defaults/account_contract.js';
import { SchnorrAccountContractArtifact } from './artifact.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures
 * verified against a Grumpkin public key stored in an immutable encrypted note.
 */
export class SchnorrAccountContract extends DefaultAccountContract {
  constructor(private signingPrivateKey: GrumpkinScalar) {
    super(SchnorrAccountContractArtifact as ContractArtifact);
  }

  getDeploymentArgs() {
    const signingPublicKey = new Schnorr().computePublicKey(this.signingPrivateKey);
    return [signingPublicKey.x, signingPublicKey.y];
  }

  getAuthWitnessProvider(_address: CompleteAddress): AuthWitnessProvider {
    return new SchnorrAuthWitnessProvider(this.signingPrivateKey);
  }
}

/** Creates auth witnesses using Schnorr signatures. */
class SchnorrAuthWitnessProvider implements AuthWitnessProvider {
  constructor(private signingPrivateKey: GrumpkinScalar) {}

  createAuthWit(messageHash: Fr): Promise<AuthWitness> {
    const schnorr = new Schnorr();
    const signature = schnorr.constructSignature(messageHash.toBuffer(), this.signingPrivateKey).toBuffer();
    return Promise.resolve(new AuthWitness(messageHash, [...signature]));
  }
}
