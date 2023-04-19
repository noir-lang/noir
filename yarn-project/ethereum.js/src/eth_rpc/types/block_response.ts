import { EthAddress } from '@aztec/foundation';
import {
  bigIntToHex,
  bufferToHex,
  hexToBigInt,
  hexToBuffer,
  hexToNumber,
  numberToHex,
} from '../../hex_string/index.js';
import {
  fromRawTransactionResponse,
  RawTransactionResponse,
  toRawTransactionResponse,
  TransactionResponse,
} from './transaction_response.js';

/**
 * Represents a raw block header response in Ethereum.
 * Contains all the essential information of a block, such as hash, parentHash, miner, and other details
 * fetched from an Ethereum node, with values encoded as hexadecimal strings.
 */
export interface RawBlockHeaderResponse {
  /**
   * The unique identifier of a block.
   */
  hash: string | null;
  /**
   * The parent block's hash value.
   */
  parentHash: string;
  /**
   * The Keccak-256 hash of the uncles data in the block.
   */
  sha3Uncles: string;
  /**
   * The Ethereum address of the block miner.
   */
  miner: string;
  /**
   * The root hash of the Ethereum state trie.
   */
  stateRoot: string;
  /**
   * The root hash of the merkle tree representing all transactions in the block.
   */
  transactionsRoot: string;
  /**
   * The root hash of the trie structure containing all transaction receipts in the block.
   */
  receiptsRoot: string;
  /**
   * Bloom filter containing logs for all transactions in the block.
   */
  logsBloom: string | null;
  /**
   * The computational effort required to mine a new block.
   */
  difficulty: string;
  /**
   * The block number in the blockchain.
   */
  number: string | null;
  /**
   * The maximum amount of gas allowed in the block.
   */
  gasLimit: string;
  /**
   * The total amount of gas consumed by all transactions in the block.
   */
  gasUsed: string;
  /**
   * Unix timestamp representing the block creation time.
   */
  timestamp: string;
  /**
   * Extra arbitrary metadata included in the block.
   */
  extraData: string;
  /**
   * A unique number used to prevent double-spending and ensure the validity of a transaction.
   */
  nonce: string | null;
  /**
   * The base fee per gas for each block, used in EIP-1559.
   */
  baseFeePerGas: string | null;
}

/**
 * Represents a raw block response in the Ethereum blockchain.
 * Contains information pertaining to the block header, transactions, and uncles in their raw hexadecimal format.
 */
export interface RawBlockResponse extends RawBlockHeaderResponse {
  /**
   * The total accumulated difficulty of the blockchain up to this block.
   */
  totalDifficulty: string;
  /**
   * Size of the block in bytes.
   */
  size: string;
  /**
   * A list of transactions included within the block.
   */
  transactions: (RawTransactionResponse | string)[];
  /**
   * An array of uncle blocks in the blockchain.
   */
  uncles: string[];
}

/**
 * Represents a block header response in the Ethereum blockchain.
 * Provides essential information about a specific block, including its hash, parent hash, miner address, and other properties.
 */
export interface BlockHeaderResponse {
  /**
   * The hash representing the unique identifier of a block.
   */
  hash: Buffer | null;
  /**
   * The hash of the parent block in the blockchain.
   */
  parentHash: Buffer;
  /**
   * The Keccak-256 hash of the uncle blocks included in the block.
   */
  sha3Uncles: Buffer;
  /**
   * The Ethereum address of the miner who successfully mined the block.
   */
  miner: EthAddress;
  /**
   * The root hash of the state trie after applying transactions.
   */
  stateRoot: Buffer;
  /**
   * The root hash of the Merkle tree containing all transaction hashes in the block.
   */
  transactionsRoot: Buffer;
  /**
   * The root hash of the Merkle tree containing transaction receipts.
   */
  receiptsRoot: Buffer;
  /**
   * A compressed representation of logs' topics and data for efficient filtering.
   */
  logsBloom: Buffer | null;
  /**
   * The computational effort required to mine a new block.
   */
  difficulty: bigint;
  /**
   * The block number within the blockchain.
   */
  number: number | null;
  /**
   * The maximum amount of gas allowed in a block.
   */
  gasLimit: number;
  /**
   * The total amount of gas consumed by all transactions in the block.
   */
  gasUsed: number;
  /**
   * The UNIX timestamp when the block was mined.
   */
  timestamp: number;
  /**
   * Arbitrary data included by the block miner.
   */
  extraData: Buffer;
  /**
   * A unique value used to prevent duplicate transactions and secure block mining.
   */
  nonce: Buffer | null;
  /**
   * The base fee per gas for the block, used in EIP-1559 transactions.
   */
  baseFeePerGas: bigint | null;
}

/**
 * Represents a block on the Ethereum blockchain.
 * Contains information about the block header, transactions, and other metadata.
 */
export interface BlockResponse<T = TransactionResponse | Buffer> extends BlockHeaderResponse {
  /**
   * The cumulative proof-of-work difficulty of the blockchain up to this block.
   */
  totalDifficulty: bigint;
  /**
   * The byte size of the block.
   */
  size: number;
  /**
   * Array of transactions included in the block.
   */
  transactions: T[];
  /**
   * Uncles are stale blocks included in the main chain to provide a reward for partially mined blocks.
   */
  uncles: string[];
}

/**
 * Convert a BlockHeaderResponse object to its raw representation (RawBlockHeaderResponse).
 * The function takes a BlockHeaderResponse containing Buffers and BigInts and converts
 * them to appropriate hex strings, preserving the structure of the original object.
 *
 * @param block - The BlockHeaderResponse object to be converted to RawBlockHeaderResponse.
 * @returns A RawBlockHeaderResponse object with hex-encoded values.
 */
export function toRawBlockHeaderResponse(block: BlockHeaderResponse): RawBlockHeaderResponse {
  return {
    hash: block.hash ? bufferToHex(block.hash) : null,
    parentHash: bufferToHex(block.parentHash),
    sha3Uncles: bufferToHex(block.sha3Uncles),
    miner: block.miner.toString(),
    stateRoot: bufferToHex(block.stateRoot),
    transactionsRoot: bufferToHex(block.transactionsRoot),
    receiptsRoot: bufferToHex(block.receiptsRoot),
    logsBloom: block.logsBloom ? bufferToHex(block.logsBloom) : null,
    difficulty: bigIntToHex(block.difficulty),
    number: block.number !== null ? numberToHex(block.number)! : null,
    gasLimit: numberToHex(block.gasLimit)!,
    gasUsed: numberToHex(block.gasUsed)!,
    timestamp: numberToHex(block.timestamp)!,
    extraData: bufferToHex(block.extraData),
    nonce: block.nonce !== null ? bufferToHex(block.nonce) : null,
    baseFeePerGas: block.baseFeePerGas !== null ? bigIntToHex(block.baseFeePerGas) : null,
  };
}

/**
 * Converts a BlockResponse object into its corresponding RawBlockResponse representation.
 * The function maps the properties in the input BlockResponse object to their respective hex-encoded string (where applicable) or appropriate raw format.
 * It handles the conversion of transaction objects as well, invoking 'toRawTransactionResponse' for each entry.
 *
 * @param block - The BlockResponse object to be converted into a RawBlockResponse.
 * @returns A RawBlockResponse object with the same data as the input BlockResponse, but in raw form.
 */
export function toRawBlockResponse(block: BlockResponse): RawBlockResponse {
  return {
    ...toRawBlockHeaderResponse(block),
    totalDifficulty: bigIntToHex(block.totalDifficulty),
    size: numberToHex(block.size)!,
    transactions: block.transactions.map(tx => (Buffer.isBuffer(tx) ? bufferToHex(tx) : toRawTransactionResponse(tx))),
    uncles: block.uncles,
  };
}

/**
 * Convert a raw block header response object to a formatted block header response object.
 * The function takes a raw block header response object with hex-encoded values and converts
 * them into appropriate data types such as Buffer, BigInt, or Number. It also converts the miner address
 * from a string to an EthAddress instance.
 *
 * @param block - The raw block header response object with hex-encoded values.
 * @returns A formatted block header response object with appropriate data types.
 */
export function fromRawBlockHeaderResponse(block: RawBlockHeaderResponse): BlockHeaderResponse {
  return {
    hash: block.hash ? hexToBuffer(block.hash) : null,
    parentHash: hexToBuffer(block.parentHash),
    sha3Uncles: hexToBuffer(block.sha3Uncles),
    miner: EthAddress.fromString(block.miner),
    stateRoot: hexToBuffer(block.stateRoot),
    transactionsRoot: hexToBuffer(block.transactionsRoot),
    receiptsRoot: hexToBuffer(block.receiptsRoot),
    logsBloom: block.logsBloom ? hexToBuffer(block.logsBloom) : null,
    difficulty: hexToBigInt(block.difficulty),
    number: block.number ? hexToNumber(block.number) : null,
    gasLimit: hexToNumber(block.gasLimit),
    gasUsed: hexToNumber(block.gasUsed),
    timestamp: hexToNumber(block.timestamp),
    extraData: hexToBuffer(block.extraData),
    nonce: block.nonce ? hexToBuffer(block.nonce) : null,
    baseFeePerGas: block.baseFeePerGas ? hexToBigInt(block.baseFeePerGas) : null,
  };
}

/**
 * Converts a RawBlockResponse object into a BlockResponse object by parsing hex-encoded strings to their corresponding types.
 * This function is useful for converting raw block responses received from external sources into a more manageable format
 * with proper data types like Buffer, EthAddress, and bigint.
 *
 * @param block - The RawBlockResponse object containing hex-encoded strings and other raw values.
 * @returns A BlockResponse object with parsed values and proper data types.
 */
export function fromRawBlockResponse(block: RawBlockResponse): BlockResponse {
  return {
    ...fromRawBlockHeaderResponse(block),
    totalDifficulty: hexToBigInt(block.totalDifficulty),
    size: hexToNumber(block.size),
    transactions: block.transactions.map(tx =>
      typeof tx === 'string' ? hexToBuffer(tx) : fromRawTransactionResponse(tx),
    ),
    uncles: block.uncles,
  };
}
