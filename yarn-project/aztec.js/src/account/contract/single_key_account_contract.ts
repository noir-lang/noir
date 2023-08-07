import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { NodeInfo, PrivateKey } from '@aztec/types';

import SchnorrSingleKeyAccountContractAbi from '../../abis/schnorr_single_key_account_contract.json' assert { type: 'json' };
import { CompleteAddress } from '../complete_address.js';
import { SingleKeyAccountEntrypoint } from '../entrypoint/single_key_account_entrypoint.js';
import { AccountContract } from './index.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures verified against
 * the note encryption key, relying on a single private key for both encryption and authentication.
 */
export class SingleKeyAccountContract implements AccountContract {
  constructor(private encryptionPrivateKey: PrivateKey) {}

  public getDeploymentArgs() {
    return Promise.resolve([]);
  }

  public async getEntrypoint({ address, partialAddress }: CompleteAddress, { chainId, version }: NodeInfo) {
    return new SingleKeyAccountEntrypoint(
      address,
      partialAddress,
      this.encryptionPrivateKey,
      await Schnorr.new(),
      chainId,
      version,
    );
  }

  public getContractAbi(): ContractAbi {
    return SchnorrSingleKeyAccountContractAbi as ContractAbi;
  }
}
