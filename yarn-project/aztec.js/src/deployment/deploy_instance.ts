import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { type ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { type Wallet } from '../wallet/index.js';
import { getDeployerContract } from './protocol_contracts.js';

/**
 * Sets up a call to the canonical deployer contract to publicly deploy a contract instance.
 * @param wallet - The wallet to use for the deployment.
 * @param instance - The instance to deploy.
 */
export function deployInstance(wallet: Wallet, instance: ContractInstanceWithAddress): ContractFunctionInteraction {
  const deployerContract = getDeployerContract(wallet);
  const { salt, contractClassId, portalContractAddress, publicKeysHash, deployer } = instance;
  const isUniversalDeploy = deployer.isZero();
  if (!isUniversalDeploy && !wallet.getAddress().equals(deployer)) {
    throw new Error(
      `Expected deployer ${deployer.toString()} does not match sender wallet ${wallet.getAddress().toString()}`,
    );
  }
  return deployerContract.methods.deploy(
    salt,
    contractClassId,
    instance.initializationHash,
    portalContractAddress,
    publicKeysHash,
    isUniversalDeploy,
  );
}
