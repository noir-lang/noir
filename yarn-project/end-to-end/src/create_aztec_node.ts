import { AztecNode } from '@aztec/aztec-node';
import { EthAddress } from '@aztec/aztec.js';

const {
  ETHEREUM_HOST = 'http://localhost:8545',
  ROLLUP_ADDRESS = '0x5FbDB2315678afecb367f032d93F642f64180aa3',
  YEETER_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
} = process.env;

export async function createAztecNode() {
  return await AztecNode.createAndSync({
    rpcUrl: ETHEREUM_HOST,
    rollupContract: EthAddress.fromString(ROLLUP_ADDRESS) as any,
    yeeterContract: EthAddress.fromString(YEETER_ADDRESS) as any,
    retryIntervalMs: 10000,
    publisherPrivateKey: Buffer.alloc(64),
    requiredConfirmations: 1,
    transactionPollingInterval: 1000,
  });
}
