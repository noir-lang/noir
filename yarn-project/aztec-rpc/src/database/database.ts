import { ContractDataSource } from '../contract_data_source/index.js';
import { TxHash } from '../tx/index.js';
import { TxDao } from './tx_dao.js';

export interface Database extends ContractDataSource {
  getTx(txHash: TxHash): Promise<TxDao | undefined>;
}
