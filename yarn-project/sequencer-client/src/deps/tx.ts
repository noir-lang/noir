import {
  AffineElement,
  AggregationObject,
  AztecAddress,
  ConstantData,
  ContractDeploymentData,
  EMITTED_EVENTS_LENGTH,
  EthAddress,
  Fq,
  Fr,
  FunctionData,
  KERNEL_L1_MSG_STACK_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  NewContractData,
  OldTreeRoots,
  OptionallyRevealedData,
  PrivateKernelPublicInputs,
  TxContext,
} from '@aztec/circuits.js';
import { AccumulatedData } from '@aztec/circuits.js';
import { UInt8Vector } from '@aztec/circuits.js';
import { Tx } from '@aztec/tx';
import times from 'lodash.times';

function frZero() {
  return Fr.fromBuffer(Buffer.alloc(32, 0));
}

function fqZero() {
  return Fq.fromBuffer(Buffer.alloc(32, 0));
}

function makeEmptyEthAddress() {
  return new EthAddress(Buffer.alloc(20, 0));
}

export function makeEmptyNewContractData(): NewContractData {
  return new NewContractData(AztecAddress.ZERO, makeEmptyEthAddress(), frZero());
}

export function makeEmptyAggregationObject(): AggregationObject {
  return new AggregationObject(
    new AffineElement(fqZero(), fqZero()),
    new AffineElement(fqZero(), fqZero()),
    times(4, frZero),
    times(6, () => 0),
  );
}

export function makeEmptyTxContext(): TxContext {
  const deploymentData = new ContractDeploymentData(frZero(), frZero(), frZero(), makeEmptyEthAddress());
  return new TxContext(false, false, true, deploymentData);
}

export function makeEmptyOldTreeRoots(): OldTreeRoots {
  return new OldTreeRoots(frZero(), frZero(), frZero(), frZero());
}

export function makeEmptyConstantData(): ConstantData {
  return new ConstantData(makeEmptyOldTreeRoots(), makeEmptyTxContext());
}

export function makeEmptyOptionallyRevealedData(): OptionallyRevealedData {
  return new OptionallyRevealedData(
    frZero(),
    new FunctionData(0, true, true),
    times(EMITTED_EVENTS_LENGTH, frZero),
    frZero(),
    makeEmptyEthAddress(),
    false,
    false,
    false,
    false,
  );
}

export function makeEmptyAccumulatedData(): AccumulatedData {
  return new AccumulatedData(
    makeEmptyAggregationObject(),
    frZero(),
    times(KERNEL_NEW_COMMITMENTS_LENGTH, frZero),
    times(KERNEL_NEW_NULLIFIERS_LENGTH, frZero),
    times(KERNEL_PRIVATE_CALL_STACK_LENGTH, frZero),
    times(KERNEL_PUBLIC_CALL_STACK_LENGTH, frZero),
    times(KERNEL_L1_MSG_STACK_LENGTH, frZero),
    times(KERNEL_NEW_CONTRACTS_LENGTH, makeEmptyNewContractData),
    times(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH, makeEmptyOptionallyRevealedData),
  );
}

function makeEmptyProof() {
  return new UInt8Vector(Buffer.alloc(0));
}

function makeEmptyPrivateKernelPublicInputs() {
  return new PrivateKernelPublicInputs(makeEmptyAccumulatedData(), makeEmptyConstantData(), true);
}

export function makeEmptyTx(): Tx {
  return new Tx(makeEmptyPrivateKernelPublicInputs(), makeEmptyProof());
}
