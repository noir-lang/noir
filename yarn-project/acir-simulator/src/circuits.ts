// See aztec3/constants.hpp
// Copied here for prototyping purposes
// In future: structured serialization?
export const ARGS_LENGTH = 8;
export const RETURN_VALUES_LENGTH = 4;
export const EMITTED_EVENTS_LENGTH = 4;

export const NEW_COMMITMENTS_LENGTH = 4;
export const NEW_NULLIFIERS_LENGTH = 4;

export const STATE_TRANSITIONS_LENGTH = 4;
export const STATE_READS_LENGTH = 4;

export const PRIVATE_CALL_STACK_LENGTH = 4;
export const PUBLIC_CALL_STACK_LENGTH = 4;
export const L1_MSG_STACK_LENGTH = 2;

export const KERNEL_NEW_COMMITMENTS_LENGTH = 16;
export const KERNEL_NEW_NULLIFIERS_LENGTH = 16;
export const KERNEL_NEW_CONTRACTS_LENGTH = 8;
export const KERNEL_PRIVATE_CALL_STACK_LENGTH = 8;
export const KERNEL_PUBLIC_CALL_STACK_LENGTH = 8;
export const KERNEL_L1_MSG_STACK_LENGTH = 4;
export const KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH = 4;

export const VK_TREE_HEIGHT = 3;
export const CONTRACT_TREE_HEIGHT = 4;
export const PRIVATE_DATA_TREE_HEIGHT = 8;
export const NULLIFIER_TREE_HEIGHT = 8;

export const PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT = 8;
export const CONTRACT_TREE_ROOTS_TREE_HEIGHT = 8;

export const FUNCTION_SELECTOR_NUM_BYTES = 31;

/**
 * Assert a member is a certain length.
 * @param obj - An object.
 * @param member - A member string.
 * @param length - The length.
 */
export function assertLength<F extends string, T extends { [f in F]: { length: number } }>(
  obj: T,
  member: F,
  length: number,
) {
  if (obj[member].length !== length) {
    throw new Error(`Expected ${member} to have length ${length}! Was: ${obj[member].length}`);
  }
}

export class Fr {
  public static ZERO = new Fr(Buffer.alloc(32));

  constructor(public readonly buffer: Buffer) {}
}

export class AztecAddress {
  public static SIZE = 64;

  public static ZERO = new AztecAddress(Buffer.alloc(AztecAddress.SIZE));

  constructor(public readonly buffer: Buffer) {}

  public equals(rhs: AztecAddress) {
    return this.buffer.equals(rhs.buffer);
  }
}

export class EthAddress {
  public static ZERO = new EthAddress(Buffer.alloc(20));

  constructor(public readonly buffer: Buffer) {}
}

/**
 * Call context.
 * @see abis/call_context.hpp
 */
export class CallContext {
  constructor(
    public msgSender: AztecAddress,
    public storageContractAddress: AztecAddress,
    public portalContractAddress: EthAddress,
    public isDelegateCall: boolean,
    public isStaticCall: boolean,
    public isContractDeployment: boolean,
  ) {}
}

/**
 * Contract deployment data in a @TxContext.
 * cpp/src/aztec3/circuits/abis/contract_deployment_data.hpp
 */
export class ContractDeploymentData {
  constructor(
    public constructorVkHash: Fr,
    public functionTreeRoot: Fr,
    public contractAddressSalt: Fr,
    public portalContractAddress: EthAddress,
  ) {}
}

/**
 * Public inputs to a private circuit.
 * @see abis/private_circuit_public_inputs.hpp.
 */
export class PrivateCircuitPublicInputs {
  constructor(
    // NOTE: Must have same order as CPP.
    public callContext: CallContext,
    public args: Fr[],
    public returnValues: Fr[],
    public emittedEvents: Fr[],
    public newCommitments: Fr[],
    public newNullifiers: Fr[],
    public privateCallStack: Fr[],
    public publicCallStack: Fr[],
    public l1MsgStack: Fr[],
    public historicPrivateDataTreeRoot: Fr,
    public historicPrivateNullifierTreeRoot: Fr,
    public historicContractTreeRoot: Fr,
    public contractDeploymentData: ContractDeploymentData,
  ) {
    assertLength(this, 'args', ARGS_LENGTH);
    assertLength(this, 'returnValues', RETURN_VALUES_LENGTH);
    assertLength(this, 'emittedEvents', EMITTED_EVENTS_LENGTH);
    assertLength(this, 'newCommitments', NEW_COMMITMENTS_LENGTH);
    assertLength(this, 'newNullifiers', NEW_NULLIFIERS_LENGTH);
    assertLength(this, 'privateCallStack', PRIVATE_CALL_STACK_LENGTH);
    assertLength(this, 'publicCallStack', PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'l1MsgStack', L1_MSG_STACK_LENGTH);
  }
}

export class TxContext {
  constructor(
    public readonly isFeePaymentTx: boolean,
    public readonly isRebatePaymentTx: boolean,
    public readonly isContractDeploymentTx: boolean,
    public readonly contractDeploymentData: ContractDeploymentData,
  ) {}
}

export interface FunctionData {
  functionSelector: Buffer;
  isSecret: boolean;
  isContructor: boolean;
}

export class TxRequest {
  constructor(
    public readonly from: AztecAddress,
    public readonly to: AztecAddress,
    public readonly functionData: FunctionData,
    public readonly args: Fr[],
    public readonly txContext: TxContext,
    public readonly nonce: Fr,
    public readonly chainId: Fr,
  ) {}

  toBuffer() {
    return Buffer.alloc(0);
  }
}

export class PrivateCallStackItem {
  constructor(
    public readonly contractAddress: AztecAddress,
    public readonly functionSelector: Buffer,
    public readonly publicInputs: PrivateCircuitPublicInputs,
  ) {}
}

export class OldTreeRoots {
  constructor(
    public privateDataTreeRoot: Fr,
    public nullifierTreeRoot: Fr,
    public contractTreeRoot: Fr,
    public privateKernelVkTreeRoot: Fr, // future enhancement
  ) {}
}

export class ConstantData {
  constructor(public oldTreeRoots: OldTreeRoots, public txContext: TxContext) {}
}

export class AggregationObject {}

export class NewContractData {
  constructor(
    public readonly contractAddress: AztecAddress,
    public readonly portalContractAddress: EthAddress,
    public readonly functionTreeRoot: Fr,
  ) {}
}

export class OptionallyRevealedData {}

export class AccumulatedTxData {
  constructor(
    public aggregationObject: AggregationObject, // Contains the aggregated proof of all previous kernel iterations

    public privateCallCount: Fr,

    public newCommitments: Fr[],
    public newNullifiers: Fr[],

    public privateCallStack: Fr[],
    public publicCallStack: Fr[],
    public l1MsgStack: Fr[],

    public newContracts: NewContractData[],

    public optionallyRevealedData: OptionallyRevealedData[],
  ) {
    assertLength(this, 'newCommitments', KERNEL_NEW_COMMITMENTS_LENGTH);
    assertLength(this, 'newNullifiers', KERNEL_NEW_NULLIFIERS_LENGTH);
    assertLength(this, 'privateCallStack', KERNEL_PRIVATE_CALL_STACK_LENGTH);
    assertLength(this, 'publicCallStack', KERNEL_PUBLIC_CALL_STACK_LENGTH);
    assertLength(this, 'l1MsgStack', KERNEL_L1_MSG_STACK_LENGTH);
    assertLength(this, 'newContracts', KERNEL_NEW_CONTRACTS_LENGTH);
    assertLength(this, 'optionallyRevealedData', KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH);
  }
}

export class PrivateKernelPublicInputs {
  constructor(public end: AccumulatedTxData, public constants: ConstantData, public isPrivateKernel: true) {}
}
