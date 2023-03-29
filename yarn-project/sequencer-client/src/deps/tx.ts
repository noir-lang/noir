import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { pedersenCompressInputs } from '@aztec/barretenberg.js/crypto';
import {
  AccumulatedData,
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
  UInt8Vector,
} from '@aztec/circuits.js';
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

function makeEmptyNewContractData(): NewContractData {
  return new NewContractData(AztecAddress.ZERO, makeEmptyEthAddress(), frZero());
}

function makeEmptyAggregationObject(): AggregationObject {
  return new AggregationObject(
    new AffineElement(fqZero(), fqZero()),
    new AffineElement(fqZero(), fqZero()),
    times(4, frZero),
    times(6, () => 0),
  );
}

function makeEmptyTxContext(): TxContext {
  const deploymentData = new ContractDeploymentData(frZero(), frZero(), frZero(), makeEmptyEthAddress());
  return new TxContext(false, false, true, deploymentData);
}

function makeEmptyOldTreeRoots(): OldTreeRoots {
  return new OldTreeRoots(frZero(), frZero(), frZero(), frZero());
}

function makeEmptyConstantData(): ConstantData {
  return new ConstantData(makeEmptyOldTreeRoots(), makeEmptyTxContext());
}

function makeEmptyOptionallyRevealedData(): OptionallyRevealedData {
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

function makeEmptyAccumulatedData(): AccumulatedData {
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

function makeEmptyUnverifiedData() {
  return Buffer.alloc(0);
}

export function makeEmptyTx(): Tx {
  const isEmpty = true;
  return new Tx(makeEmptyPrivateKernelPublicInputs(), makeEmptyProof(), makeEmptyUnverifiedData(), isEmpty);
}

export function hashNewContractData(wasm: BarretenbergWasm, cd: NewContractData) {
  return pedersenCompressInputs(
    wasm,
    [cd.contractAddress, cd.portalContractAddress, cd.functionTreeRoot].map(x => x.toBuffer()),
  );
}
