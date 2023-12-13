import { AcirSimulator } from '@aztec/acir-simulator';
import { KeyStore, StateInfoProvider } from '@aztec/types';

import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { PxeDatabase } from '../database/pxe_database.js';
import { SimulatorOracle } from '../simulator_oracle/index.js';

/**
 * Helper method to create an instance of the acir simulator.
 */
export function getAcirSimulator(
  db: PxeDatabase,
  stateInfoProvider: StateInfoProvider,
  keyStore: KeyStore,
  contractDataOracle?: ContractDataOracle,
) {
  const simulatorOracle = new SimulatorOracle(
    contractDataOracle ?? new ContractDataOracle(db, stateInfoProvider),
    db,
    keyStore,
    stateInfoProvider,
  );
  return new AcirSimulator(simulatorOracle);
}
