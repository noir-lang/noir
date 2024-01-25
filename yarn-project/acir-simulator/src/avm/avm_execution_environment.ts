import { GlobalVariables } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

/**
 * Contains variables that remain constant during AVM execution
 * These variables are provided by the public kernel circuit
 */
export class AvmExecutionEnvironment {
  constructor(
    /** - */
    public readonly address: AztecAddress,
    /** - */
    public readonly storageAddress: AztecAddress,
    /** - */
    public readonly origin: AztecAddress,
    /** - */
    public readonly sender: AztecAddress,
    /** - */
    public readonly portal: EthAddress,
    /** - */
    public readonly feePerL1Gas: Fr,
    /** - */
    public readonly feePerL2Gas: Fr,
    /** - */
    public readonly feePerDaGas: Fr,
    /** - */
    public readonly contractCallDepth: Fr,
    /** - */
    public readonly globals: GlobalVariables,
    /** - */
    public readonly isStaticCall: boolean,
    /** - */
    public readonly isDelegateCall: boolean,
    /** - */
    public readonly calldata: Fr[],
  ) {}
}
