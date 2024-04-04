// All code in this file needs to die once the public executor is phased out.
import { UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  ContractStorageRead,
  ContractStorageUpdateRequest,
  type GlobalVariables,
  L2ToL1Message,
  type ReadRequest,
  SideEffect,
  SideEffectLinkedToNoteHash,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { createSimulationError } from '../common/errors.js';
import { type PublicExecution, type PublicExecutionResult } from '../public/execution.js';
import { AvmExecutionEnvironment } from './avm_execution_environment.js';
import { type AvmContractCallResults } from './avm_message_call_result.js';
import { type JournalData } from './journal/journal.js';
import { Mov } from './opcodes/memory.js';

/** Temporary Method
 *
 * Convert a PublicExecution(Environment) object to an AvmExecutionEnvironment
 *
 * @param current
 * @param globalVariables
 * @returns
 */
export function temporaryCreateAvmExecutionEnvironment(
  current: PublicExecution,
  globalVariables: GlobalVariables,
): AvmExecutionEnvironment {
  // Function selector is included temporarily until noir codegens public contract bytecode in a single blob
  return new AvmExecutionEnvironment(
    current.contractAddress,
    current.callContext.storageContractAddress,
    current.callContext.msgSender, // TODO: origin is not available
    current.callContext.msgSender,
    current.callContext.portalContractAddress,
    /*feePerL1Gas=*/ Fr.zero(),
    /*feePerL2Gas=*/ Fr.zero(),
    /*feePerDaGas=*/ Fr.zero(),
    /*contractCallDepth=*/ Fr.zero(),
    globalVariables,
    current.callContext.isStaticCall,
    current.callContext.isDelegateCall,
    current.args,
    current.functionData.selector,
  );
}

/** Temporary Method
 *
 * Convert the result of an AVM contract call to a PublicExecutionResult for the public kernel
 *
 * @param execution
 * @param newWorldState
 * @param result
 * @returns
 */
export function temporaryConvertAvmResults(
  execution: PublicExecution,
  newWorldState: JournalData,
  result: AvmContractCallResults,
): PublicExecutionResult {
  const newNoteHashes = newWorldState.newNoteHashes.map(noteHash => new SideEffect(noteHash, Fr.zero()));

  const contractStorageReads: ContractStorageRead[] = [];
  const reduceStorageReadRequests = (contractAddress: bigint, storageReads: Map<bigint, Fr[]>) => {
    return storageReads.forEach((innerArray, key) => {
      innerArray.forEach(value => {
        contractStorageReads.push(new ContractStorageRead(new Fr(key), new Fr(value), 0));
      });
    });
  };
  newWorldState.storageReads.forEach((storageMap: Map<bigint, Fr[]>, address: bigint) =>
    reduceStorageReadRequests(address, storageMap),
  );

  const contractStorageUpdateRequests: ContractStorageUpdateRequest[] = [];
  const reduceStorageUpdateRequests = (contractAddress: bigint, storageUpdateRequests: Map<bigint, Fr[]>) => {
    return storageUpdateRequests.forEach((innerArray, key) => {
      innerArray.forEach(value => {
        contractStorageUpdateRequests.push(new ContractStorageUpdateRequest(new Fr(key), new Fr(value), 0));
      });
    });
  };
  newWorldState.storageWrites.forEach((storageMap: Map<bigint, Fr[]>, address: bigint) =>
    reduceStorageUpdateRequests(address, storageMap),
  );

  const returnValues = result.output;

  // TODO(follow up in pr tree): NOT SUPPORTED YET, make sure hashing and log resolution is done correctly
  // Disabled.
  const nestedExecutions: PublicExecutionResult[] = [];
  const nullifierReadRequests: ReadRequest[] = [];
  const nullifierNonExistentReadRequests: ReadRequest[] = [];
  const newNullifiers: SideEffectLinkedToNoteHash[] = newWorldState.newNullifiers.map(
    (nullifier, i) => new SideEffectLinkedToNoteHash(nullifier.toField(), Fr.zero(), new Fr(i + 1)),
  );
  const unencryptedLogs = UnencryptedFunctionL2Logs.empty();
  const newL2ToL1Messages = newWorldState.newL1Messages.map(() => L2ToL1Message.empty());
  // TODO keep track of side effect counters
  const startSideEffectCounter = Fr.ZERO;
  const endSideEffectCounter = Fr.ZERO;

  return {
    execution,
    nullifierReadRequests,
    nullifierNonExistentReadRequests,
    newNoteHashes,
    newL2ToL1Messages,
    startSideEffectCounter,
    endSideEffectCounter,
    newNullifiers,
    contractStorageReads,
    contractStorageUpdateRequests,
    returnValues,
    nestedExecutions,
    unencryptedLogs,
    reverted: result.reverted,
    revertReason: result.revertReason ? createSimulationError(result.revertReason) : undefined,
  };
}

export function isAvmBytecode(bytecode: Buffer): boolean {
  const magicBuf = Buffer.from([
    Mov.opcode, // opcode
    0x00, // indirect
    ...Buffer.from('000018ca', 'hex'), // srcOffset
    ...Buffer.from('000018ca', 'hex'), // dstOffset
  ]);
  const magicSize = magicBuf.length;
  return bytecode.subarray(-magicSize).equals(magicBuf);
}
