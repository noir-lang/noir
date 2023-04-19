import { CallRequest, EthereumRpc, TxHash } from '../eth_rpc/index.js';
import { hexToBuffer } from '../hex_string/index.js';
import { ContractAbi } from './abi/contract_abi.js';

/**
 * Represents a decoded error from a contract execution.
 * Contains optional name and params properties, as well as a mandatory message property
 * providing a human-readable description of the error.
 */
export interface DecodedError {
  /**
   * The name of the decoded error.
   */
  name?: string;

  /**
   * An array of decoded error parameters.
   */
  params?: any[];
  /**
   * A human-readable description of the error.
   */
  message: string;
}

/**
 * Given a ContractAbi and a data buffer containing the return data of a failed call(), will attempt to lookup the
 * error in the contracts abi and to decode the return data.
 */
export function decodeErrorFromContract(contractAbi: ContractAbi, data: Buffer) {
  const sigHash = data.subarray(0, 4);
  const args = data.subarray(4);

  const error = contractAbi.errors.find(e => e.signature.equals(sigHash));
  if (!error) {
    return;
  }
  const decoded = error.decodeParameters(args);
  const params = Array.from({ length: decoded.__length__ }).map((_, i) => decoded[i]);

  const errorValue: DecodedError = {
    name: error.name!,
    params,
    message: `${error.name!}(${params.map(p => p.toString()).join(',')})`,
  };

  return errorValue;
}

/**
 * If a transaction fails, you can call this to decode the error.
 * It locates the block within which the tx failed, and then "replays" the tx via a call against the state of the prior
 * block. This should reproduce the error deterministically, but will return data that can be decoded to show the error.
 */
export async function decodeErrorFromContractByTxHash(
  contractAbi: ContractAbi,
  txHash: TxHash,
  ethRpc: EthereumRpc,
): Promise<DecodedError> {
  const { from, gas, to, maxFeePerGas, maxPriorityFeePerGas, value, input, blockNumber } =
    await ethRpc.getTransactionByHash(txHash);
  const callReq: CallRequest = {
    from,
    gas,
    to: to!,
    maxFeePerGas,
    maxPriorityFeePerGas,
    value,
    data: input,
  };
  try {
    await ethRpc.call(callReq, blockNumber! - 1);
    // We expect the call the throw.
    return { message: 'Cannot determine failure, as call succeeded when replaying transaction.' };
  } catch (err: any) {
    if (err.data && err.data.length > 2) {
      const decodedError = decodeErrorFromContract(contractAbi, hexToBuffer(err.data));
      if (decodedError) {
        return decodedError;
      }
    }
    return { message: err.message };
  }
}
