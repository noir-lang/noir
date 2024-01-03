import { PartialAddress } from '@aztec/circuits.js';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractArtifact } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { AuthWitness, CompleteAddress, GrumpkinPrivateKey } from '@aztec/types';

import { AuthWitnessProvider } from '../account/interface.js';
import { generatePublicKey } from '../utils/index.js';
import SchnorrSingleKeyAccountContractArtifact from './artifacts/SchnorrSingleKeyAccount.json' assert { type: 'json' };
import { BaseAccountContract } from './base_account_contract.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures verified against
 * the note encryption key, relying on a single private key for both encryption and authentication.
 */
export class SingleKeyAccountContract extends BaseAccountContract {
  constructor(private encryptionPrivateKey: GrumpkinPrivateKey) {
    super(SchnorrSingleKeyAccountContractArtifact as ContractArtifact);
  }

  getDeploymentArgs(): any[] {
    return [];
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

  createAuthWitness(message: Fr): Promise<AuthWitness> {
    const schnorr = new Schnorr();
    const signature = schnorr.constructSignature(message.toBuffer(), this.privateKey);
    const publicKey = generatePublicKey(this.privateKey);
    const witness = [...publicKey.toFields(), ...signature.toBuffer(), this.partialAddress];
    return Promise.resolve(new AuthWitness(message, witness));
  }
}
