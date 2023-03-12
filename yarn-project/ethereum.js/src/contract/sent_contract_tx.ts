import { EthereumRpc, LogResponse, TransactionReceipt, TxHash } from '../eth_rpc/index.js';
import { SentTransaction } from '../eth_rpc/sent_tx.js';
import { ContractAbi } from './abi/contract_abi.js';
import { ContractTxReceipt } from './contract_tx_receipt.js';
import { decodeErrorFromContractByTxHash } from './decode_error.js';

/**
 * Extends the generic eth_rpc SentTransaction class, to provide handling of the contract function call receipt.
 * It decodes the events and extends the receipt with three new properties:
 * - `anonymousLogs`: Logs that were emitted as part of external contract calls (unknown to this contracts abi).
 * - `events`: A mapping from EventName to an object with named arguments to useful value types.
 * - `error`: An optional error object `{ message: string, decodedError?: DecodedError }`.
 */
export class SentContractTx extends SentTransaction {
  constructor(ethRpc: EthereumRpc, protected contractAbi: ContractAbi, promise: Promise<TxHash>) {
    super(ethRpc, promise);
  }

  protected async handleReceipt(throwOnError = true, receipt: TransactionReceipt) {
    const result = await handleReceipt(receipt, this.contractAbi, this.ethRpc);
    if (result.error && throwOnError) {
      throw new Error(`Receipt indicates transaction failed: ${result.error.message}`);
    }
    return result;
  }
}

export async function handleReceipt(
  receipt: TransactionReceipt,
  contractAbi: ContractAbi,
  ethRpc: EthereumRpc,
): Promise<ContractTxReceipt> {
  const { logs, to, contractAddress = to!, status, transactionHash } = receipt;

  if (!status) {
    const error = await decodeErrorFromContractByTxHash(contractAbi, transactionHash, ethRpc);
    return { ...receipt, anonymousLogs: [], events: {}, error };
  }

  if (!Array.isArray(logs)) {
    return { ...receipt, anonymousLogs: [], events: {} };
  }

  const isAnonymous = (log: LogResponse) => !log.address.equals(contractAddress) || !contractAbi.findEntryForLog(log);

  const anonymousLogs = logs.filter(isAnonymous);

  const events = logs.reduce((a, log) => {
    if (isAnonymous(log)) {
      return a;
    }
    const ev = contractAbi.decodeEvent(log);
    a[ev.event] = a[ev.event] || [];
    a[ev.event].push(ev);
    return a;
  }, {});

  return { ...receipt, anonymousLogs, events };
}
