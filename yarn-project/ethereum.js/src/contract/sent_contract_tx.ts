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

  /**
   * Processes the transaction receipt by decoding events, handling errors, and populating anonymous logs.
   * The function takes a TransactionReceipt, a ContractAbi, and an EthereumRpc as its input parameters,
   * and returns a Promise that resolves to a ContractTxReceipt containing the processed data.
   * It separates logs into anonymousLogs (emitted by external contract calls) and decodes events
   * according to the contract's ABI. If the transaction status is falsy, it attempts to decode
   * the error using the provided ABI and includes it in the resulting ContractTxReceipt.
   *
   * @param receipt - The TransactionReceipt to be processed.
   * @param contractAbi - The ContractAbi instance used for event decoding and error handling.
   * @param ethRpc - The EthereumRpc instance used for interacting with the Ethereum network.
   * @returns A Promise that resolves to a ContractTxReceipt object containing anonymousLogs, decoded events, and optional error information.
   */
  protected async handleReceipt(throwOnError = true, receipt: TransactionReceipt) {
    const result = await handleReceipt(receipt, this.contractAbi, this.ethRpc);
    if (result.error && throwOnError) {
      throw new Error(`Receipt indicates transaction failed: ${result.error.message}`);
    }
    return result;
  }
}

/**
 * Handle and process a contract transaction receipt by decoding the events and extending
 * the original receipt with additional properties (anonymousLogs, events, error) related
 * to the contract function call. This function filters out anonymous logs that were emitted
 * as part of external contract calls and decodes known events based on the provided ABI.
 * If the receipt indicates a failure, it also attempts to decode the error message.
 *
 * @param receipt - The TransactionReceipt object for the contract function call.
 * @param contractAbi - The ContractAbi instance representing the contract's ABI.
 * @param ethRpc - The EthereumRpc instance for making RPC calls.
 * @returns A Promise resolving to a ContractTxReceipt object with decoded events and extended properties.
 */
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
