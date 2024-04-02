import { generatePublicKey } from '@aztec/aztec.js';
import { type AuthWitnessProvider } from '@aztec/aztec.js/account';
import { AuthWitness, type CompleteAddress, type GrumpkinPrivateKey } from '@aztec/circuit-types';
import { type PartialAddress } from '@aztec/circuits.js';
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
  constructor(private encryptionPrivateKey: GrumpkinPrivateKey) {
    super(SchnorrSingleKeyAccountContractArtifact as ContractArtifact);
  }

  getDeploymentArgs(): undefined {
    return undefined;
  }

  getAuthWitnessProvider({ partialAddress }: CompleteAddress): AuthWitnessProvider {
    return new SingleKeyAuthWitnessProvider(this.encryptionPrivateKey, partialAddress);
  }
}

/**
 * Creates auth witnesses using Schnorr signatures and including the partial address and public key
 * in the witness, so verifiers do not need to store the public key and can instead validate it
 * by reconstructing the current address.
 */
class SingleKeyAuthWitnessProvider implements AuthWitnessProvider {
  constructor(private privateKey: GrumpkinPrivateKey, private partialAddress: PartialAddress) {}

  createAuthWit(messageHash: Fr): Promise<AuthWitness> {
    const schnorr = new Schnorr();
    const signature = schnorr.constructSignature(messageHash.toBuffer(), this.privateKey);
    const publicKey = generatePublicKey(this.privateKey);
    const witness = [...publicKey.toFields(), ...signature.toBuffer(), this.partialAddress];
    return Promise.resolve(new AuthWitness(messageHash, witness));
  }
}
