// All code in this file needs to die once the public executor is phased out in favor of the AVM.
import { type GasSettings, type GlobalVariables, type Header } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { promisify } from 'util';
import { gunzip } from 'zlib';

import { AvmExecutionEnvironment } from '../avm/avm_execution_environment.js';
import { Mov } from '../avm/opcodes/memory.js';
import { type PublicExecution } from './execution.js';

/**
 * Convert a PublicExecution(Environment) object to an AvmExecutionEnvironment
 *
 * @param current
 * @param globalVariables
 * @returns
 */
export function createAvmExecutionEnvironment(
  current: PublicExecution,
  header: Header,
  globalVariables: GlobalVariables,
  gasSettings: GasSettings,
  transactionFee: Fr,
): AvmExecutionEnvironment {
  return new AvmExecutionEnvironment(
    current.contractAddress,
    current.callContext.storageContractAddress,
    current.callContext.msgSender,
    globalVariables.gasFees.feePerL2Gas,
    globalVariables.gasFees.feePerDaGas,
    /*contractCallDepth=*/ Fr.zero(),
    header,
    globalVariables,
    current.callContext.isStaticCall,
    current.callContext.isDelegateCall,
    current.args,
    gasSettings,
    transactionFee,
    current.functionSelector,
  );
}

const AVM_MAGIC_SUFFIX = Buffer.from([
  Mov.opcode, // opcode
  0x00, // indirect
  ...Buffer.from('000018ca', 'hex'), // srcOffset
  ...Buffer.from('000018ca', 'hex'), // dstOffset
]);

export function markBytecodeAsAvm(bytecode: Buffer): Buffer {
  return Buffer.concat([bytecode, AVM_MAGIC_SUFFIX]);
}

// This is just a helper function for the AVM circuit.
export async function decompressBytecodeIfCompressed(bytecode: Buffer): Promise<Buffer> {
  try {
    return await promisify(gunzip)(bytecode);
  } catch {
    // If the bytecode is not compressed, the gunzip call will throw an error
    // In this case, we assume the bytecode is not compressed and continue.
    return Promise.resolve(bytecode);
  }
}

export async function isAvmBytecode(bytecode: Buffer): Promise<boolean> {
  const decompressedBytecode = await decompressBytecodeIfCompressed(bytecode);
  const magicSize = AVM_MAGIC_SUFFIX.length;
  return decompressedBytecode.subarray(-magicSize).equals(AVM_MAGIC_SUFFIX);
}
