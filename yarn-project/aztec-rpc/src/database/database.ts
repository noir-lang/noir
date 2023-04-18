import { AztecAddress, Fr, Point } from '@aztec/foundation';
import { TxHash } from '@aztec/types';
import { ContractDatabase } from '../contract_database/index.js';
import { TxAuxDataDao } from './tx_aux_data_dao.js';
import { TxDao } from './tx_dao.js';

export interface Database extends ContractDatabase {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
  getTxsByAddress(from: AztecAddress): Promise<TxDao[]>;
  addTx(tx: TxDao): Promise<void>;
  addTxs(txs: TxDao[]): Promise<void>;

  getTxAuxData(contract: AztecAddress, storageSlot: Fr): Promise<TxAuxDataDao[]>;
  addTxAuxData(txAuxDataDao: TxAuxDataDao): Promise<void>;
  addTxAuxDataBatch(txAuxDataDaos: TxAuxDataDao[]): Promise<void>;
  removeNullifiedTxAuxData(nullifiers: Fr[], account: Point): Promise<TxAuxDataDao[]>;
}
