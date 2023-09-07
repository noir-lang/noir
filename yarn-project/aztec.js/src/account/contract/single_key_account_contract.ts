import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress, GrumpkinPrivateKey, NodeInfo } from '@aztec/types';

import SchnorrSingleKeyAccountContractAbi from '../../abis/schnorr_single_key_account_contract.json' assert { type: 'json' };
import { SingleKeyAccountEntrypoint } from '../entrypoint/single_key_account_entrypoint.js';
import { AccountContract } from './index.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures verified against
 * the note encryption key, relying on a single private key for both encryption and authentication.
 */
export class SingleKeyAccountContract implements AccountContract {
  constructor(private encryptionPrivateKey: GrumpkinPrivateKey) {}

  public getDeploymentArgs() {
    return Promise.resolve([]);
  }

  public getEntrypoint({ address, partialAddress }: CompleteAddress, { chainId, version }: NodeInfo) {
    return Promise.resolve(
      new SingleKeyAccountEntrypoint(address, partialAddress, this.encryptionPrivateKey, chainId, version),
    );
  }

  public getContractAbi(): ContractAbi {
    return SchnorrSingleKeyAccountContractAbi as unknown as ContractAbi;
  }
}
