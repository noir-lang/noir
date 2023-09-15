import { Fr } from '@aztec/circuits.js';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { AuthWitness, CompleteAddress, GrumpkinPrivateKey } from '@aztec/types';

import SchnorrAccountContractAbi from '../../abis/schnorr_account_contract.json' assert { type: 'json' };
import { AuthWitnessProvider } from '../interface.js';
import { BaseAccountContract } from './base_account_contract.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures
 * verified against a Grumpkin public key stored in an immutable encrypted note.
 */
export class SchnorrAccountContract extends BaseAccountContract {
  constructor(private signingPrivateKey: GrumpkinPrivateKey) {
    super(SchnorrAccountContractAbi as unknown as ContractAbi);
  }

  async getDeploymentArgs() {
    const signingPublicKey = await Schnorr.new().then(e => e.computePublicKey(this.signingPrivateKey));
    return [signingPublicKey.x, signingPublicKey.y];
  }

  getAuthWitnessProvider(_address: CompleteAddress): AuthWitnessProvider {
    return new SchnorrAuthWitnessProvider(this.signingPrivateKey);
  }
}

/** Creates auth witnesses using Schnorr signatures. */
class SchnorrAuthWitnessProvider implements AuthWitnessProvider {
  constructor(private signingPrivateKey: GrumpkinPrivateKey) {}

  async createAuthWitness(message: Fr): Promise<AuthWitness> {
    const schnorr = await Schnorr.new();
    const signature = schnorr.constructSignature(message.toBuffer(), this.signingPrivateKey).toBuffer();
    return new AuthWitness(message, [...signature]);
  }
}
