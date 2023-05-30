import { AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { DeployL1Contracts, deployL1Contracts } from '@aztec/ethereum';
import { mnemonicToAccount } from 'viem/accounts';
import { MNEMONIC, localAnvil } from './fixtures.js';
import { AztecAddress, AztecRPCServer, createAztecRPCServer } from '@aztec/aztec.js';

/**
 * Sets up the environment for the end-to-end tests.
 * @param numberOfAccounts - The number of new accounts to be created once the RPC server is initiated.
 */
export async function setup(numberOfAccounts = 1): Promise<{
  /**
   * The Aztec Node service.
   */
  aztecNode: AztecNodeService;
  /**
   * The Aztec RPC server.
   */
  aztecRpcServer: AztecRPCServer;
  /**
   * Return values from deployL1Contracts function.
   */
  deployL1ContractsValues: DeployL1Contracts;
  /**
   * The accounts created by the RPC server.
   */
  accounts: AztecAddress[];
  /**
   * The Aztec Node configuration.
   */
  config: AztecNodeConfig;
  /**
   * Logger instance named as the current test.
   */
  logger: DebugLogger;
}> {
  const config = getConfigEnvVars();

  const describeBlockName = expect.getState().currentTestName?.split(' ')[0];

  const logger = createDebugLogger('aztec:' + describeBlockName);

  const hdAccount = mnemonicToAccount(MNEMONIC);
  const privKey = hdAccount.getHdKey().privateKey;
  const deployL1ContractsValues = await deployL1Contracts(config.rpcUrl, hdAccount, localAnvil, logger);

  config.publisherPrivateKey = Buffer.from(privKey!);
  config.rollupContract = deployL1ContractsValues.rollupAddress;
  config.unverifiedDataEmitterContract = deployL1ContractsValues.unverifiedDataEmitterAddress;
  config.inboxContract = deployL1ContractsValues.inboxAddress;

  const aztecNode = await AztecNodeService.createAndSync(config);
  const aztecRpcServer = await createAztecRPCServer(aztecNode);
  for (let i = 0; i < numberOfAccounts; ++i) {
    await aztecRpcServer.addAccount();
  }

  const accounts = await aztecRpcServer.getAccounts();

  return {
    aztecNode,
    aztecRpcServer,
    deployL1ContractsValues,
    accounts,
    config,
    logger,
  };
}
