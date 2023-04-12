import { AztecNode, AztecNodeConfig } from '@aztec/aztec-node';
import { EthAddress } from '@aztec/foundation';

export const createAztecNode = async (
  rollupContract: EthAddress,
  unverifiedDataEmitterContract: EthAddress,
  rpcUrl: string,
  publisherPrivateKey: Buffer,
) => {
  const config: AztecNodeConfig = {
    rollupContract,
    unverifiedDataEmitterContract,
    rpcUrl,
    publisherPrivateKey,
    retryIntervalMs: 1000,
    requiredConfirmations: 1,
    transactionPollingInterval: 1000,
    archiverPollingInterval: 1000,
    maxTxsPerBlock: 4,
  };
  return await AztecNode.createAndSync(config);
};
