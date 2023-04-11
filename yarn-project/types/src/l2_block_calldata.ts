import { AppendOnlyTreeSnapshot } from '@aztec/circuits.js';
import { ContractData } from './contract_data.js';

/**
 * The fixed size data that makes up the rollup header.
 */
export type L2BlockCalldataHeader = {
  /**
   * The id of the rollup.
   * Similar to the block number in Ethereum.
   */
  rollupId: number;
  /**
   * The tree snapshot of the private data tree at the start of the rollup.
   */
  startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the nullifier tree at the start of the rollup.
   */
  startNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the contract tree at the start of the rollup.
   */
  startContractTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic private data tree roots at the start of the rollup.
   */
  startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic contract tree roots at the start of the rollup.
   */
  startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the private data tree at the end of the rollup.
   * By using start and end, we know the number of new commitments in the rollup.
   */
  endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the nullifier tree at the end of the rollup.
   * By using start and end, we know the number of new nullifiers in the rollup.
   */
  endNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the contract tree at the end of the rollup.
   */
  endContractTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic private data tree roots at the end of the rollup.
   */
  endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic contract tree roots at the end of the rollup.
   */
  endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot;
};

/**
 * The data that makes up the rollup calldata.
 */
export type L2BlockCalldata = {
  /**
   * The header of the rollup calldata.
   */
  header: L2BlockCalldataHeader;
  /**
   * The commitments to be inserted into the private data tree.
   * The commitments are field elements, and 4 commitments are inserted for each kernel proof.
   */
  newCommitments: Buffer[];
  /**
   * The nullifiers to be inserted into the nullifier tree.
   * The nullifiers are field elements, and 4 nullifiers are inserted for each kernel proof.
   */
  newNullifiers: Buffer[];
  /**
   * The contracts leafs to be inserted into the contract tree.
   * The contracts are field element, there can be at most 1 contract deployed for each kernel proof.
   */
  newContracts: Buffer[];
  /**
   * The aztec_address and eth_address for the deployed contract and its portal contract.
   * The aztec_address is the address of the deployed contract, will be a field element (32 bytes)
   * The eth_address will be the address of the portal contract, or address(0) if no portal is used (20 bytes).
   */
  newContractData: ContractData[];
};
