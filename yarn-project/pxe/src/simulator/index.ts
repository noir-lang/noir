import { type AztecNode, type KeyStore } from '@aztec/circuit-types';
import { AcirSimulator } from '@aztec/simulator';

import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { type PxeDatabase } from '../database/pxe_database.js';
import { SimulatorOracle } from '../simulator_oracle/index.js';

/**
 * Helper method to create an instance of the acir simulator.
 */
export function getAcirSimulator(
  db: PxeDatabase,
  aztecNode: AztecNode,
  keyStore: KeyStore,
  contractDataOracle?: ContractDataOracle,
) {
  const simulatorOracle = new SimulatorOracle(
    contractDataOracle ?? new ContractDataOracle(db),
    db,
    keyStore,
    aztecNode,
  );
  return new AcirSimulator(simulatorOracle, aztecNode);
}
