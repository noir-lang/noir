import { AztecNode, AztecNodeConfig } from '@aztec/aztec-node';
import { EthAddress } from '@aztec/ethereum.js/eth_address';

export const createAztecNode = async (
  rollupContract: EthAddress,
  yeeterContract: EthAddress,
  rpcUrl: string,
  publisherPrivateKey: Buffer,
) => {
  const config: AztecNodeConfig = {
    rollupContract,
    yeeterContract,
    rpcUrl,
    publisherPrivateKey,
    retryIntervalMs: 1000,
    requiredConfirmations: 1,
    transactionPollingInterval: 1000,
    archiverPollingInterval: 1000,
  };
  return await AztecNode.createAndSync(config);
};
