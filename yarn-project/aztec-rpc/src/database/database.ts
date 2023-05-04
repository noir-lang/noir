import { TxHash } from '@aztec/types';
import { ContractDatabase } from '../contract_database/index.js';
import { TxAuxDataDao } from './tx_aux_data_dao.js';
import { TxDao } from './tx_dao.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { MerkleTreeId } from '@aztec/types';

export interface Database extends ContractDatabase {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
  getTxsByAddress(from: AztecAddress): Promise<TxDao[]>;
  addTx(tx: TxDao): Promise<void>;
  addTxs(txs: TxDao[]): Promise<void>;

  getTxAuxData(contract: AztecAddress, storageSlot: Fr): Promise<TxAuxDataDao[]>;
  addTxAuxData(txAuxDataDao: TxAuxDataDao): Promise<void>;
  addTxAuxDataBatch(txAuxDataDaos: TxAuxDataDao[]): Promise<void>;
  removeNullifiedTxAuxData(nullifiers: Fr[], account: Point): Promise<TxAuxDataDao[]>;

  getTreeRoots(): Promise<Record<MerkleTreeId, Fr>>;
  setTreeRoots(roots: Record<MerkleTreeId, Fr>): Promise<void>;
}
