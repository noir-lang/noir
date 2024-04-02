import { type PublicKey } from '@aztec/circuit-types';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Point } from '@aztec/foundation/fields';

import { type Wallet } from '../account/index.js';
import { ContractBase } from './contract_base.js';
import { DeployMethod } from './deploy_method.js';

/**
 * The Contract class represents a contract and provides utility methods for interacting with it.
 * It enables the creation of ContractFunctionInteraction instances for each function in the contract's ABI,
 * allowing users to call or send transactions to these functions. Additionally, the Contract class can be used
 * to attach the contract instance to a deployed contract on-chain through the PXE, which facilitates
 * interaction with Aztec's privacy protocol.
 */
export class Contract extends ContractBase {
  /**
   * Creates a contract instance.
   * @param address - The deployed contract's address.
   * @param artifact - Build artifact of the contract.
   * @param wallet - The wallet to use when interacting with the contract.
   * @param portalContract - The portal contract address on L1, if any.
   * @returns A promise that resolves to a new Contract instance.
   */
  public static async at(address: AztecAddress, artifact: ContractArtifact, wallet: Wallet): Promise<Contract> {
    const instance = await wallet.getContractInstance(address);
    if (instance === undefined) {
      throw new Error(`Contract instance at ${address.toString()} has not been registered in the wallet's PXE`);
    }
    return new Contract(instance, artifact, wallet);
  }

  /**
   * Creates a tx to deploy a new instance of a contract.
   * @param wallet - The wallet for executing the deployment.
   * @param artifact - Build artifact of the contract to deploy
   * @param args - Arguments for the constructor.
   * @param constructorName - The name of the constructor function to call.
   */
  public static deploy(wallet: Wallet, artifact: ContractArtifact, args: any[], constructorName?: string) {
    const postDeployCtor = (address: AztecAddress, wallet: Wallet) => Contract.at(address, artifact, wallet);
    return new DeployMethod(Point.ZERO, wallet, artifact, postDeployCtor, args, constructorName);
  }

  /**
   * Creates a tx to deploy a new instance of a contract using the specified public key to derive the address.
   * @param publicKey - Public key for deriving the address.
   * @param wallet - The wallet for executing the deployment.
   * @param artifact - Build artifact of the contract.
   * @param args - Arguments for the constructor.
   * @param constructorName - The name of the constructor function to call.
   */
  public static deployWithPublicKey(
    publicKey: PublicKey,
    wallet: Wallet,
    artifact: ContractArtifact,
    args: any[],
    constructorName?: string,
  ) {
    const postDeployCtor = (address: AztecAddress, wallet: Wallet) => Contract.at(address, artifact, wallet);
    return new DeployMethod(publicKey, wallet, artifact, postDeployCtor, args, constructorName);
  }
}
