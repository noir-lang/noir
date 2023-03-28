import { TxHash } from '@aztec/tx';

import { ContractDataSource } from '../contract_data_source/index.js';
import { TxDao } from './tx_dao.js';

export interface Database extends ContractDataSource {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
}
