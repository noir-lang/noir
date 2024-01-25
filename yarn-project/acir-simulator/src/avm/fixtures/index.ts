// Place large AVM text fixtures in here
import { GlobalVariables } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { AvmExecutionEnvironment } from '../avm_execution_environment.js';

/**
 * An interface that allows to override the default values of the AvmExecutionEnvironment
 */
export interface AvmExecutionEnvironmentOverrides {
  /** - */
  address?: AztecAddress;
  /** - */
  storageAddress?: AztecAddress;
  /** - */
  origin?: AztecAddress;
  /** - */
  sender?: AztecAddress;
  /** - */
  portal?: EthAddress;
  /** - */
  feePerL1Gas?: Fr;
  /** - */
  feePerL2Gas?: Fr;
  /** - */
  feePerDaGas?: Fr;
  /** - */
  contractCallDepth?: Fr;
  /** - */
  globals?: GlobalVariables;
  /** - */
  isStaticCall?: boolean;
  /** - */
  isDelegateCall?: boolean;
  /** - */
  calldata?: Fr[];
}

/**
 * Create an empty instance of the Execution Environment where all values are zero, unless overriden in the overrides object
 */
export function initExecutionEnvironment(overrides?: AvmExecutionEnvironmentOverrides): AvmExecutionEnvironment {
  return new AvmExecutionEnvironment(
    overrides?.address ?? AztecAddress.zero(),
    overrides?.storageAddress ?? AztecAddress.zero(),
    overrides?.origin ?? AztecAddress.zero(),
    overrides?.sender ?? AztecAddress.zero(),
    overrides?.portal ?? EthAddress.ZERO,
    overrides?.feePerL1Gas ?? Fr.zero(),
    overrides?.feePerL2Gas ?? Fr.zero(),
    overrides?.feePerDaGas ?? Fr.zero(),
    overrides?.contractCallDepth ?? Fr.zero(),
    overrides?.globals ?? GlobalVariables.empty(),
    overrides?.isStaticCall ?? false,
    overrides?.isDelegateCall ?? false,
    overrides?.calldata ?? [],
  );
}
