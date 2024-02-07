import { ContractInstanceWithAddress } from '@aztec/types/contracts';

import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { Wallet } from '../wallet/index.js';
import { getDeployerContract } from './protocol_contracts.js';

/**
 * Sets up a call to the canonical deployer contract to publicly deploy a contract instance.
 * @param wallet - The wallet to use for the deployment.
 * @param instance - The instance to deploy.
 * @param opts - Additional options.
 */
export function deployInstance(
  wallet: Wallet,
  instance: ContractInstanceWithAddress,
  opts: { /** Set to true to *not* mix in the deployer into the address. */ universalDeploy?: boolean } = {},
): ContractFunctionInteraction {
  const deployer = getDeployerContract(wallet);
  const { salt, contractClassId, portalContractAddress, publicKeysHash } = instance;
  return deployer.methods.deploy(
    salt,
    contractClassId,
    instance.initializationHash,
    portalContractAddress,
    publicKeysHash,
    !!opts.universalDeploy,
  );
}
