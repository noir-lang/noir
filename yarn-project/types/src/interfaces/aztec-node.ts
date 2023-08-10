import { EthAddress } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import {
  ContractCommitmentProvider,
  ContractData,
  ContractDataAndBytecode,
  DataCommitmentProvider,
  L1ToL2MessageProvider,
  L2Block,
  L2BlockL2Logs,
  LogType,
  MerkleTreeId,
  Tx,
  TxHash,
} from '../index.js';

/**
 * The aztec node.
 * We will probably implement the additional interfaces by means other than Aztec Node as it's currently a privacy leak
 */
export interface AztecNode extends DataCommitmentProvider, L1ToL2MessageProvider, ContractCommitmentProvider {
  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  isReady(): Promise<boolean>;

  /**
   * Get the a given block.
   * @param number - The block number being requested.
   * @returns The blocks requested.
   */
  getBlock(number: number): Promise<L2Block | undefined>;

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param limit - The maximum number of blocks to return.
   * @returns The blocks requested.
   */
  getBlocks(from: number, limit: number): Promise<L2Block[]>;

  /**
   * Method to fetch the current block height.
   * @returns The block height as a number.
   */
  getBlockHeight(): Promise<number>;

  /**
   * Method to fetch the version of the rollup the node is connected to.
   * @returns The rollup version.
   */
  getVersion(): Promise<number>;

  /**
   * Method to fetch the chain id of the base-layer for the rollup.
   * @returns The chain id.
   */
  getChainId(): Promise<number>;

  /**
   * Method to fetch the rollup contract address at the base-layer.
   * @returns The rollup address.
   */
  getRollupAddress(): Promise<EthAddress>;

  /**
   * Lookup the L2 contract data for this contract.
   * Contains the ethereum portal address and bytecode.
   * @param contractAddress - The contract data address.
   * @returns The complete contract data including portal address & bytecode (if we didn't throw an error).
   */
  getContractDataAndBytecode(contractAddress: AztecAddress): Promise<ContractDataAndBytecode | undefined>;

  /**
   * Lookup the contract data for this contract.
   * Contains the ethereum portal address .
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The maximum number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  getLogs(from: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]>;

  /**
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   * @returns Nothing.
   */
  sendTx(tx: Tx): Promise<void>;

  /**
   * Method to retrieve pending txs.
   * @returns The pending txs.
   */
  getPendingTxs(): Promise<Tx[]>;

  /**
   * Method to retrieve a single pending tx.
   * @param txHash - The transaction hash to return.
   * @returns The pending tx if it exists.
   */
  getPendingTxByHash(txHash: TxHash): Promise<Tx | undefined>;

  /**
   * Gets the storage value at the given contract slot. Our version of eth_getStorageAt.
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot (or undefined if not found).
   */
  getPublicStorageAt(contract: AztecAddress, slot: bigint): Promise<Buffer | undefined>;

  /**
   * Returns the current committed roots for the data trees.
   * @returns The current committed roots for the data trees.
   */
  getTreeRoots(): Promise<Record<MerkleTreeId, Fr>>;
}
