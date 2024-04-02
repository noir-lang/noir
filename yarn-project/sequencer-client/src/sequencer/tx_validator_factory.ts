import { type AztecAddress, type EthAddress, type Fr, type GlobalVariables } from '@aztec/circuits.js';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { WorldStateDB, WorldStatePublicDB } from '../simulator/public_executor.js';
import { TxValidator } from './tx_validator.js';

export class TxValidatorFactory {
  constructor(
    private merkleTreeDb: MerkleTreeOperations,
    private contractDataSource: ContractDataSource,
    private gasPortalAddress: EthAddress,
  ) {}

  buildTxValidator(
    globalVariables: GlobalVariables,
    allowedFeePaymentContractClasses: Fr[],
    allowedFeePaymentContractInstances: AztecAddress[],
  ): TxValidator {
    return new TxValidator(
      new WorldStateDB(this.merkleTreeDb),
      new WorldStatePublicDB(this.merkleTreeDb),
      this.contractDataSource,
      globalVariables,
      {
        allowedFeePaymentContractClasses,
        allowedFeePaymentContractInstances,
        gasPortalAddress: this.gasPortalAddress,
      },
    );
  }
}
