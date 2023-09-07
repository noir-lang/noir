import { Ecdsa } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress, NodeInfo } from '@aztec/types';

import EcdsaAccountContractAbi from '../../abis/ecdsa_account_contract.json' assert { type: 'json' };
import { StoredKeyAccountEntrypoint } from '../entrypoint/stored_key_account_entrypoint.js';
import { AccountContract } from './index.js';

/**
 * Account contract that authenticates transactions using ECDSA signatures
 * verified against a secp256k1 public key stored in an immutable encrypted note.
 */ export class EcdsaAccountContract implements AccountContract {
  constructor(private signingPrivateKey: Buffer) {}

  public async getDeploymentArgs() {
    const signingPublicKey = await Ecdsa.new().then(e => e.computePublicKey(this.signingPrivateKey));
    return [signingPublicKey.subarray(0, 32), signingPublicKey.subarray(32, 64)];
  }

  public async getEntrypoint({ address }: CompleteAddress, { chainId, version }: NodeInfo) {
    const ecdsa = await Ecdsa.new();
    const signClosure = (msg: Buffer) => ecdsa.constructSignature(msg, this.signingPrivateKey);
    return new StoredKeyAccountEntrypoint(address, signClosure, chainId, version);
  }

  public getContractAbi(): ContractAbi {
    return EcdsaAccountContractAbi as unknown as ContractAbi;
  }
}
