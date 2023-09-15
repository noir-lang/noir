import { Fr, PartialAddress } from '@aztec/circuits.js';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { AuthWitness, CompleteAddress, GrumpkinPrivateKey } from '@aztec/types';

import SchnorrSingleKeyAccountContractAbi from '../../abis/schnorr_single_key_account_contract.json' assert { type: 'json' };
import { generatePublicKey } from '../../index.js';
import { AuthWitnessProvider } from '../interface.js';
import { BaseAccountContract } from './base_account_contract.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures verified against
 * the note encryption key, relying on a single private key for both encryption and authentication.
 */
export class SingleKeyAccountContract extends BaseAccountContract {
  constructor(private encryptionPrivateKey: GrumpkinPrivateKey) {
    super(SchnorrSingleKeyAccountContractAbi as unknown as ContractAbi);
  }

  getDeploymentArgs(): Promise<any[]> {
    return Promise.resolve([]);
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

  async createAuthWitness(message: Fr): Promise<AuthWitness> {
    const schnorr = await Schnorr.new();
    const signature = schnorr.constructSignature(message.toBuffer(), this.privateKey);
    const publicKey = await generatePublicKey(this.privateKey);
    const witness = [...publicKey.toFields(), ...signature.toBuffer(), this.partialAddress];
    return new AuthWitness(message, witness);
  }
}
