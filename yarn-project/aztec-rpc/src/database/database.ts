import { TxHash } from '@aztec/tx';

import { ContractDataSource } from '../contract_database/index.js';
import { TxDao } from './tx_dao.js';

export interface Database extends ContractDataSource {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
  addOrUpdateTx(tx: TxDao): Promise<void>;
}
