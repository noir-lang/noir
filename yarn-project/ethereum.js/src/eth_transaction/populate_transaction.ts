import { EthereumRpc } from '../eth_rpc/ethereum_rpc.js';
import { EthAccount } from '../eth_account/index.js';
import { EthTransaction } from './eth_transaction.js';
import { BlockResponse } from '../eth_rpc/index.js';

/**
 * Populate a partial EthTransaction by querying node for defaults.
 * `to` can be optional when creating a new contract.
 * `data` is optional when just doing value sends.
 * `gas` must be provided.
 * Everything else should be populated.
 */
export async function populateTransaction(
  tx: Partial<EthTransaction>,
  privateKey: Buffer,
  eth: EthereumRpc,
): Promise<EthTransaction> {
  if (!tx.gas) {
    throw new Error('gas is missing or 0.');
  }

  // Get the missing info from the Ethereum node.
  const promises: [Promise<number>, Promise<BlockResponse | undefined> | undefined, Promise<number>] = [
    tx.chainId === undefined ? eth.getChainId() : Promise.resolve(tx.chainId),
    tx.maxFeePerGas === undefined ? eth.getBlockByNumber('latest') : undefined,
    tx.nonce === undefined ? eth.getTransactionCount(new EthAccount(privateKey).address) : Promise.resolve(tx.nonce),
  ];

  const [chainId, block, nonce] = await Promise.all(promises);

  const maxPriorityFeePerGas = tx.maxPriorityFeePerGas !== undefined ? tx.maxPriorityFeePerGas : BigInt(2500000000); // 2.5 gwei
  const maxFeePerGas = block
    ? (block.baseFeePerGas! * BigInt(115)) / BigInt(100) + maxPriorityFeePerGas
    : tx.maxFeePerGas!;
  const { to, gas, value = BigInt(0), data } = tx;

  return {
    chainId,
    to,
    gas,
    maxFeePerGas,
    maxPriorityFeePerGas,
    value,
    data,
    nonce,
  };
}
