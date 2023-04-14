import { ACVMField, toACVMField } from './acvm.js';
import { Fr } from '@aztec/foundation';
import {
  CallContext,
  ContractDeploymentData,
  FunctionData,
  OldTreeRoots,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  TxContext,
} from '@aztec/circuits.js';
import { NoteLoadOracleInputs } from '../db_oracle.js';

// Utilities to write TS classes to ACVM Field arrays
// In the order that the ACVM expects them

export function toACVMFunctionData(functionData: FunctionData): ACVMField[] {
  return [
    toACVMField(functionData.functionSelector),
    toACVMField(functionData.isPrivate),
    toACVMField(functionData.isConstructor),
  ];
}

export function toACVMCallContext(callContext: CallContext): ACVMField[] {
  return [
    toACVMField(callContext.isContractDeployment),
    toACVMField(callContext.isDelegateCall),
    toACVMField(callContext.isStaticCall),
    toACVMField(callContext.msgSender),
    toACVMField(callContext.portalContractAddress),
    toACVMField(callContext.storageContractAddress),
  ];
}

export function toACVMContractDeploymentData(contractDeploymentData: ContractDeploymentData): ACVMField[] {
  return [
    toACVMField(contractDeploymentData.constructorVkHash),
    toACVMField(contractDeploymentData.functionTreeRoot),
    toACVMField(contractDeploymentData.contractAddressSalt),
    toACVMField(contractDeploymentData.portalContractAddress),
  ];
}

export function toACVMPublicInputs(publicInputs: PrivateCircuitPublicInputs): ACVMField[] {
  return [
    ...toACVMCallContext(publicInputs.callContext),

    ...publicInputs.args.map(toACVMField),
    ...publicInputs.returnValues.map(toACVMField),
    ...publicInputs.emittedEvents.map(toACVMField),
    ...publicInputs.newCommitments.map(toACVMField),
    ...publicInputs.newNullifiers.map(toACVMField),
    ...publicInputs.privateCallStack.map(toACVMField),
    ...publicInputs.publicCallStack.map(toACVMField),
    ...publicInputs.l1MsgStack.map(toACVMField),

    toACVMField(publicInputs.historicPrivateDataTreeRoot),
    toACVMField(publicInputs.historicPrivateNullifierTreeRoot),
    toACVMField(publicInputs.historicContractTreeRoot),

    ...toACVMContractDeploymentData(publicInputs.contractDeploymentData),
  ];
}

export function toAcvmCallPrivateStackItem(item: PrivateCallStackItem): ACVMField[] {
  return [
    toACVMField(item.contractAddress),
    ...toACVMFunctionData(item.functionData),
    ...toACVMPublicInputs(item.publicInputs),
  ];
}

export function toAcvmNoteLoadOracleInputs(
  noteLoadOracleInputs: NoteLoadOracleInputs,
  privateDataTreeRoot: Fr,
): ACVMField[] {
  return [
    ...noteLoadOracleInputs.preimage.map(f => toACVMField(f)),
    toACVMField(noteLoadOracleInputs.index),
    ...noteLoadOracleInputs.siblingPath.map(f => toACVMField(f)),
    toACVMField(privateDataTreeRoot),
  ];
}

// We still need this function until we can get user-defined ordering of structs for fn arguments
// TODO When that is sorted out on noir side, we can use instead the utilities in this file
export function writeInputs(
  args: Fr[],
  callContext: CallContext,
  txContext: TxContext,
  oldRoots: OldTreeRoots,
  witnessStartIndex = 1,
) {
  const fields = [
    ...args.map(arg => toACVMField(arg)),

    toACVMField(callContext.isContractDeployment),
    toACVMField(callContext.isDelegateCall),
    toACVMField(callContext.isStaticCall),
    toACVMField(callContext.msgSender),
    toACVMField(callContext.portalContractAddress),
    toACVMField(callContext.storageContractAddress),

    toACVMField(txContext.contractDeploymentData.constructorVkHash),
    toACVMField(txContext.contractDeploymentData.contractAddressSalt),
    toACVMField(txContext.contractDeploymentData.functionTreeRoot),
    toACVMField(false),
    toACVMField(txContext.contractDeploymentData.portalContractAddress),

    toACVMField(oldRoots.contractTreeRoot),
    toACVMField(oldRoots.nullifierTreeRoot),
    toACVMField(oldRoots.privateDataTreeRoot),
  ];

  return fields.reduce((witness, field, index) => {
    witness.set(index + witnessStartIndex, field);
    return witness;
  }, new Map<number, ACVMField>());
}
