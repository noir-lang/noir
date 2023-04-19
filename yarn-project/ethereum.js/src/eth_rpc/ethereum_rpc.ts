import { EthAddress } from '@aztec/foundation';
import { EthSignature } from '../eth_sign/index.js';
import { TypedData } from '../eth_typed_data/index.js';
import { hexToBuffer } from '../hex_string/index.js';
import { EthereumProvider } from '../provider/index.js';
import { SentTx, SentTransaction } from './sent_tx.js';
import { TxHash } from './tx_hash.js';
import {
  CallRequest,
  fromRawBlockResponse,
  fromRawLogResponse,
  fromRawTransactionReceipt,
  fromRawTransactionResponse,
  NumberOrTag,
  numberOrTagToHex,
  RawLogResponse,
  toRawCallRequest,
  toRawEstimateRequest,
  toRawTransactionRequest,
  TransactionRequest,
} from './types/index.js';
import { LogRequest, toRawLogRequest } from './types/log_request.js';

/**
 * Provides a direct 1 to 1 mapping with the ethereum JSON-RPC specification.
 * Link - https://ethereum.org/en/developers/docs/apis/json-rpc.
 *
 * Types are marshalled to/from sensible types.
 * Number.
 * BigInt.
 * Buffer.
 * TxHash.
 * EthAddress.
 */
export class EthereumRpc {
  constructor(private provider: EthereumProvider) {}

  /**
   * Retrieves the currently implemented Ethereum protocol version.
   * The returned value follows the semantic versioning specification (semver).
   * This may be useful for determining compatibility between client and server implementations.
   *
   * @returns A Promise that resolves to a number representing the Ethereum protocol version.
   */
  public async protocolVersion() {
    const result = await this.provider.request({ method: 'eth_protocolVersion' });
    return Number(result);
  }

  /**
   * Retrieves the syncing status of the connected Ethereum node.
   * When the node is not syncing, returns false. If syncing, returns an object containing
   * startingBlock, currentBlock, and highestBlock which represent the block numbers of
   * the syncing process's start point, current progress, and end point respectively.
   *
   * @returns A Promise that resolves to either false or an object with syncing status data.
   */
  public async syncing() {
    const result = await this.provider.request({ method: 'eth_syncing' });
    return {
      startingBlock: Number(result.startingBlock),
      currentBlock: Number(result.currentBlock),
      highestBlock: Number(result.highestBlock),
    };
  }

  /**
   * Retrieve the currently configured chain ID of the Ethereum network.
   * The chain ID is a unique identifier for each Ethereum network, allowing clients to distinguish
   * between multiple networks and ensuring transaction replay protection.
   *
   * @returns A number representing the current chain ID.
   */
  public async getChainId() {
    const result = await this.provider.request({ method: 'eth_chainId' });
    return Number(result);
  }

  /**
   * Retrieve the contract code of a given `address` at a specific block number or block tag.
   * The function allows querying the Ethereum blockchain for smart contract bytecode.
   * Results are returned as a Buffer containing the bytecode in hexadecimal format.
   *
   * @param address - The EthAddress instance representing the Ethereum address of the smart contract.
   * @param numberOrTag - Optional parameter specifying the block number or block tag (default is 'latest').
   * @returns A Promise that resolves to a Buffer containing the contract code in hexadecimal format.
   */
  public async getCode(address: EthAddress, numberOrTag: NumberOrTag = 'latest') {
    const result = await this.provider.request({
      method: 'eth_getCode',
      params: [address.toString(), numberOrTagToHex(numberOrTag)],
    });
    return hexToBuffer(result);
  }

  /**
   * Retrieves a list of Ethereum accounts available on the connected node.
   * Each account is represented by an EthAddress instance. Useful for
   * querying and interacting with accounts managed by the connected Ethereum node.
   *
   * @returns An array of EthAddress instances representing the available accounts.
   */
  public async getAccounts() {
    const result: string[] = await this.provider.request({
      method: 'eth_accounts',
    });
    return result.map(EthAddress.fromString);
  }

  /**
   * Retrieves the number of transactions sent from the specified Ethereum address.
   * This function sends a request to the 'eth_getTransactionCount' method with the address and 'latest' as parameters.
   * The response is then converted to a Number before being returned.
   *
   * @param addr - The EthAddress instance representing the Ethereum address.
   * @returns The number of transactions sent from the given address.
   */
  public async getTransactionCount(addr: EthAddress) {
    const result = await this.provider.request({
      method: 'eth_getTransactionCount',
      params: [addr.toString(), 'latest'],
    });
    return Number(result);
  }

  /**
   * Retrieves the current balance of the given Ethereum address.
   * The balance is returned as a BigInt representing the amount of wei held by the address.
   *
   * @param addr - The EthAddress instance representing the Ethereum address to fetch the balance for.
   * @returns A BigInt representing the balance of the given address in wei.
   */
  public async getBalance(addr: EthAddress) {
    const result = await this.provider.request({
      method: 'eth_getBalance',
      params: [addr.toString(), 'latest'],
    });
    return BigInt(result);
  }

  /**
   * Retrieves a transaction by its hash.
   * The function returns the transaction details such as block hash, block number, from/to addresses, etc., by querying the Ethereum blockchain.
   *
   * @param txHash - The transaction hash of the target transaction.
   * @returns A Promise that resolves to the transaction details in a human-readable format.
   */
  public async getTransactionByHash(txHash: TxHash) {
    const result = await this.provider.request({
      method: 'eth_getTransactionByHash',
      params: [txHash.toString()],
    });
    return fromRawTransactionResponse(result);
  }

  /**
   * Retrieves the transaction receipt for the given transaction hash.
   * The transaction receipt contains information about the execution of a transaction,
   * such as the status, gas used, and logs generated by the transaction.
   * Returns undefined if the transaction hash is not found or the transaction is pending.
   *
   * @param txHash - The TxHash object representing the transaction hash.
   * @returns A Promise that resolves to an object containing transaction receipt details or undefined if not found.
   */
  public async getTransactionReceipt(txHash: TxHash) {
    const result = await this.provider.request({
      method: 'eth_getTransactionReceipt',
      params: [txHash.toString()],
    });
    return fromRawTransactionReceipt(result);
  }

  /**
   * Retrieves the current block number from the Ethereum blockchain.
   * The result is a decimal representation of the latest mined block number.
   *
   * @returns A Promise that resolves to the current block number as a Number.
   */
  public async blockNumber() {
    const result = await this.provider.request({
      method: 'eth_blockNumber',
      params: [],
    });
    return Number(result);
  }

  /**
   * Retrieve the block information by its block number or block tag.
   * The block information includes data such as block hash, parent hash, miner, difficulty, etc.
   * The transactions in the block can be optionally fetched in full detail by setting 'fullTxs' to true.
   *
   * @param numberOrTag - The block number or block tag ('earliest', 'latest', 'pending') to fetch the block information for.
   * @param fullTxs - If set to true, the block information will include full transaction details. Defaults to false.
   * @returns An object containing the detailed block information or undefined if the block is not found.
   */
  public async getBlockByNumber(numberOrTag: NumberOrTag, fullTxs = false) {
    const result = await this.provider.request({
      method: 'eth_getBlockByNumber',
      params: [numberOrTagToHex(numberOrTag), fullTxs],
    });
    return result ? fromRawBlockResponse(result) : undefined;
  }

  /**
   * Sign a message with the specified Ethereum address using the 'eth_sign' JSON-RPC method.
   * The resulting signature can be used to verify that the message was signed by the owner of the provided address.
   *
   * @param address - The EthAddress instance representing the Ethereum address to sign the message with.
   * @param message - The Buffer instance representing the message to be signed.
   * @returns A Promise that resolves to an EthSignature instance representing the signed message.
   */
  public async sign(address: EthAddress, message: Buffer) {
    const result = await this.provider.request({
      method: 'eth_sign',
      params: [address.toString(), `0x${message.toString('hex')}`],
    });
    return EthSignature.fromString(result);
  }

  /**
   * Sign a message using the 'personal_sign' Ethereum JSON-RPC method with a specified Ethereum address.
   * The provided message should be a Buffer, and the Ethereum address should be an EthAddress instance.
   * This method is commonly used for signing messages to verify the ownership of an Ethereum account.
   *
   * @param message - The message to be signed as a Buffer.
   * @param address - The EthAddress instance representing the Ethereum account used to sign the message.
   * @returns An EthSignature instance containing the signed message from the provided Ethereum address.
   */
  public async personalSign(message: Buffer, address: EthAddress) {
    const result = await this.provider.request({
      method: 'personal_sign',
      params: [`0x${message.toString('hex')}`, address.toString()],
    });
    return EthSignature.fromString(result);
  }

  /**
   * Sign typed data using the EIP-712 Typed Data v4 standard.
   * This function sends an 'eth_signTypedData_v4' JSON-RPC request to the Ethereum provider
   * with the given address and data. The result is a signature in hex format which can be
   * used to verify the signer of the typed data.
   *
   * @param address - The EthAddress of the signer.
   * @param data - The TypedData object representing the structured data to be signed.
   * @returns An EthSignature instance representing the signed message.
   */
  public async signTypedDataV4(address: EthAddress, data: TypedData) {
    const result = await this.provider.request({
      method: 'eth_signTypedData_v4',
      params: [address.toString(), data],
    });
    return EthSignature.fromString(result);
  }

  /**
   * Estimate the amount of gas needed for a given transaction to succeed.
   * The 'estimateGas' function sends a simulated call request to the Ethereum network
   * and returns the estimated gas required for the transaction to be executed.
   * This is useful for determining the gas limit to set when sending a real transaction,
   * helping to ensure that transactions are executed successfully and avoiding
   * unnecessary gas consumption.
   *
   * @param tx - The CallRequest object containing transaction details.
   * @returns A number representing the estimated gas required for the transaction.
   */
  public async estimateGas(tx: CallRequest) {
    const result = await this.provider.request({
      method: 'eth_estimateGas',
      params: [toRawEstimateRequest(tx)],
    });
    return Number(result);
  }

  /**
   * Executes a call to a smart contract function without modifying the blockchain state.
   * The 'call' function returns the result of the executed code, and any gas used during execution is not deducted from the account.
   * Useful for querying information from the blockchain or testing if a transaction would succeed without actually sending it.
   *
   * @param tx - The transaction request object containing information such as 'from', 'to', 'gas', 'gasPrice', 'value', and 'data'.
   * @param numberOrTag - (Optional) A block number or one of the string tags "latest", "earliest", or "pending". Default is "latest".
   * @returns A Buffer containing the return value of the executed function, if any.
   */
  public async call(tx: CallRequest, numberOrTag: NumberOrTag = 'latest') {
    const data = await this.provider.request({
      method: 'eth_call',
      params: [toRawCallRequest(tx), numberOrTagToHex(numberOrTag)],
    });
    return Buffer.from(data.slice(2), 'hex');
  }

  /**
   * Sends a signed transaction to the Ethereum network and returns a SentTx instance.
   * The 'sendTransaction' method takes a TransactionRequest object as input, converts it to a raw transaction,
   * and sends it to the network using the 'eth_sendTransaction' JSON-RPC method. It then returns a SentTx instance
   * which can be used to track the status of the submitted transaction, such as fetching the transaction receipt.
   *
   * @param tx - The TransactionRequest object containing the details of the transaction to be sent.
   * @returns A SentTx instance for tracking the submitted transaction's status.
   */
  public sendTransaction(tx: TransactionRequest): SentTx {
    const txHashPromise = this.provider.request({
      method: 'eth_sendTransaction',
      params: [toRawTransactionRequest(tx)],
    });
    return new SentTransaction(this, txHashPromise);
  }

  /**
   * Sign a given Ethereum transaction using the specified private key.
   * The 'TransactionRequest' object contains necessary information to sign the transaction such as nonce, gas price, gas limit, etc.
   * Returns a Promise that resolves with a Buffer containing the signed transaction in raw bytes format.
   *
   * @param tx - The TransactionRequest object containing information required to sign the transaction.
   * @returns A Promise that resolves with a raw signed transaction in buffer format.
   */
  public async signTransaction(tx: TransactionRequest) {
    const result = await this.provider.request({
      method: 'eth_signTransaction',
      params: [toRawTransactionRequest(tx)],
    });
    return hexToBuffer(result);
  }

  /**
   * Retrieve logs from Ethereum nodes based on the log request provided.
   * This function queries the Ethereum node using JSON-RPC and returns an array of logs matching
   * the given filters specified in the logRequest object.
   *
   * @param logRequest - An object containing the filter parameters for the logs to be retrieved.
   * @returns An array of log objects matching the filter parameters.
   */
  public async getLogs(logRequest: LogRequest) {
    const result: RawLogResponse[] = await this.provider.request({
      method: 'eth_getLogs',
      params: [toRawLogRequest(logRequest)],
    });
    return result.map(fromRawLogResponse);
  }
}
