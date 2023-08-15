import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress, NodeInfo, PrivateKey } from '@aztec/types';

import SchnorrAccountContractAbi from '../../abis/schnorr_account_contract.json' assert { type: 'json' };
import { StoredKeyAccountEntrypoint } from '../entrypoint/stored_key_account_entrypoint.js';
import { AccountContract } from './index.js';

/**
 * Account contract that authenticates transactions using Schnorr signatures
 * verified against a Grumpkin public key stored in an immutable encrypted note.
 */
export class SchnorrAccountContract implements AccountContract {
  constructor(private signingPrivateKey: PrivateKey) {}

  public async getDeploymentArgs() {
    const signingPublicKey = await Schnorr.new().then(e => e.computePublicKey(this.signingPrivateKey));
    return [signingPublicKey.x, signingPublicKey.y];
  }

  public async getEntrypoint({ address }: CompleteAddress, { chainId, version }: NodeInfo) {
    return new StoredKeyAccountEntrypoint(address, this.signingPrivateKey, await Schnorr.new(), chainId, version);
  }

  public getContractAbi(): ContractAbi {
    return SchnorrAccountContractAbi as ContractAbi;
  }
}
