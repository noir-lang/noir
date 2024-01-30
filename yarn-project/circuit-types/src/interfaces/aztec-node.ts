import { Header } from '@aztec/circuits.js';
import { L1ContractAddresses } from '@aztec/ethereum';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { ContractData, ExtendedContractData } from '../contract_data.js';
import { L2Block } from '../l2_block.js';
import { L2Tx } from '../l2_tx.js';
import { GetUnencryptedLogsResponse, L2BlockL2Logs, LogFilter, LogType } from '../logs/index.js';
import { Tx, TxHash } from '../tx/index.js';
import { SequencerConfig } from './configs.js';
import { StateInfoProvider } from './state_info_provider.js';

/**
 * The aztec node.
 * We will probably implement the additional interfaces by means other than Aztec Node as it's currently a privacy leak
 */
export interface AztecNode extends StateInfoProvider {
  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  isReady(): Promise<boolean>;

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param limit - The maximum number of blocks to return.
   * @returns The blocks requested.
   */
  getBlocks(from: number, limit: number): Promise<L2Block[]>;

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
   * Method to fetch the currently deployed l1 contract addresses.
   * @returns The deployed contract addresses.
   */
  getL1ContractAddresses(): Promise<L1ContractAddresses>;

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined>;

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
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse>;

  /**
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   * @returns Nothing.
   */
  sendTx(tx: Tx): Promise<void>;

  /**
   * Get a settled tx.
   * @param txHash - The txHash being requested.
   * @returns The tx requested.
   */
  getTx(txHash: TxHash): Promise<L2Tx | undefined>;

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
   * Gets the storage value at the given contract storage slot.
   *
   * @remarks The storage slot here refers to the slot as it is defined in Noir not the index in the merkle tree.
   * Aztec's version of `eth_getStorageAt`.
   *
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot.
   */
  getPublicStorageAt(contract: AztecAddress, slot: Fr): Promise<Fr>;

  /**
   * Returns the currently committed block header.
   * @returns The current committed block header.
   */
  getHeader(): Promise<Header>;

  /**
   * Simulates the public part of a transaction with the current state.
   * This currently just checks that the transaction execution succeeds.
   * @param tx - The transaction to simulate.
   **/
  simulatePublicCalls(tx: Tx): Promise<void>;

  /**
   * Updates the configuration of this node.
   * @param config - Updated configuration to be merged with the current one.
   */
  setConfig(config: Partial<SequencerConfig>): Promise<void>;
}
