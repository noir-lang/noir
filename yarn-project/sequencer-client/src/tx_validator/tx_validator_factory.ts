import { type AllowedFunction, type ProcessedTx, type Tx } from '@aztec/circuit-types';
import { type EthAddress, type GlobalVariables } from '@aztec/circuits.js';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { WorldStateDB, WorldStatePublicDB } from '../simulator/public_executor.js';
import { AggregateTxValidator } from './aggregate_tx_validator.js';
import { DoubleSpendTxValidator } from './double_spend_validator.js';
import { GasTxValidator } from './gas_validator.js';
import { MetadataTxValidator } from './metadata_validator.js';
import { PhasesTxValidator } from './phases_validator.js';
import { type TxValidator } from './tx_validator.js';

export class TxValidatorFactory {
  constructor(
    private merkleTreeDb: MerkleTreeOperations,
    private contractDataSource: ContractDataSource,
    private gasPortalAddress: EthAddress,
  ) {}

  validatorForNewTxs(
    globalVariables: GlobalVariables,
    setupAllowList: AllowedFunction[],
    teardownAllowList: AllowedFunction[],
  ): TxValidator<Tx> {
    return new AggregateTxValidator(
      new MetadataTxValidator(globalVariables),
      new DoubleSpendTxValidator(new WorldStateDB(this.merkleTreeDb)),
      new PhasesTxValidator(this.contractDataSource, setupAllowList, teardownAllowList),
      new GasTxValidator(new WorldStatePublicDB(this.merkleTreeDb), getCanonicalGasTokenAddress(this.gasPortalAddress)),
    );
  }

  validatorForProcessedTxs(): TxValidator<ProcessedTx> {
    return new DoubleSpendTxValidator(new WorldStateDB(this.merkleTreeDb));
  }
}
