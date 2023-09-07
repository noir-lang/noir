import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress, GrumpkinPrivateKey, NodeInfo } from '@aztec/types';

import AuthWitnessAccountContractAbi from '../../abis/schnorr_auth_witness_account_contract.json' assert { type: 'json' };
import { AuthWitnessAccountEntrypoint } from '../entrypoint/auth_witness_account_entrypoint.js';
import { AccountContract } from './index.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures verified against
 * the note encryption key, relying on a single private key for both encryption and authentication.
 * Extended to pull verification data from the oracle instead of passed as arguments.
 */
export class AuthWitnessAccountContract implements AccountContract {
  constructor(private encryptionPrivateKey: GrumpkinPrivateKey) {}

  public getDeploymentArgs() {
    return Promise.resolve([]);
  }

  public async getEntrypoint({ address, partialAddress }: CompleteAddress, { chainId, version }: NodeInfo) {
    return new AuthWitnessAccountEntrypoint(
      address,
      partialAddress,
      this.encryptionPrivateKey,
      await Schnorr.new(),
      chainId,
      version,
    );
  }

  public getContractAbi(): ContractAbi {
    return AuthWitnessAccountContractAbi as unknown as ContractAbi;
  }
}
