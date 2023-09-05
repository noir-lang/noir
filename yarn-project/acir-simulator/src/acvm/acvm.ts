import { FunctionDebugMetadata, OpcodeLocation } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { NoirCallStack, SourceCodeLocation } from '@aztec/types';

import {
  ExecutionError,
  ForeignCallInput,
  ForeignCallOutput,
  WasmBlackBoxFunctionSolver,
  WitnessMap,
  executeCircuitWithBlackBoxSolver,
} from 'acvm_js';

import { traverseCauseChain } from '../common/errors.js';

/**
 * The format for fields on the ACVM.
 */
export type ACVMField = string;
/**
 * The format for witnesses of the ACVM.
 */
export type ACVMWitness = WitnessMap;

export const ZERO_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES)}`;
export const ONE_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES - 1)}01`;

/**
 * The supported oracle names.
 */
type ORACLE_NAMES =
  | 'computeSelector'
  | 'packArguments'
  | 'getSecretKey'
  | 'getNote'
  | 'getNotes'
  | 'getRandomField'
  | 'notifyCreatedNote'
  | 'notifyNullifiedNote'
  | 'callPrivateFunction'
  | 'callPublicFunction'
  | 'enqueuePublicFunctionCall'
  | 'storageRead'
  | 'storageWrite'
  | 'getCommitment'
  | 'getL1ToL2Message'
  | 'getPortalContractAddress'
  | 'emitEncryptedLog'
  | 'emitUnencryptedLog'
  | 'getPublicKey'
  | 'debugLog'
  | 'debugLogWithPrefix';

/**
 * A type that does not require all keys to be present.
 */
type PartialRecord<K extends keyof any, T> = Partial<Record<K, T>>;

/**
 * The callback interface for the ACIR.
 */
export type ACIRCallback = PartialRecord<ORACLE_NAMES, (...args: ForeignCallInput[]) => Promise<ForeignCallOutput>>;

/**
 * The result of executing an ACIR.
 */
export interface ACIRExecutionResult {
  /**
   * The partial witness of the execution.
   */
  partialWitness: ACVMWitness;
}

/**
 * Extracts the call stack from the location of a failing opcode and the debug metadata.
 * One opcode can point to multiple calls due to inlining.
 */
function getSourceCodeLocationsFromOpcodeLocation(
  opcodeLocation: string,
  debug: FunctionDebugMetadata,
): SourceCodeLocation[] {
  const { debugSymbols, files } = debug;

  const callStack = debugSymbols.locations[opcodeLocation] || [];
  return callStack.map(call => {
    const { file: fileId, span } = call;

    const { path, source } = files[fileId];

    const locationText = source.substring(span.start, span.end + 1);
    const precedingText = source.substring(0, span.start);
    const previousLines = precedingText.split('\n');
    // Lines and columns in stacks are one indexed.
    const line = previousLines.length;
    const column = previousLines[previousLines.length - 1].length + 1;

    return {
      filePath: path,
      line,
      column,
      fileSource: source,
      locationText,
    };
  });
}

/**
 * Extracts the source code locations for an array of opcode locations
 * @param opcodeLocations - The opcode locations that caused the error.
 * @param debug - The debug metadata of the function.
 * @returns The source code locations.
 */
export function resolveOpcodeLocations(
  opcodeLocations: OpcodeLocation[],
  debug: FunctionDebugMetadata,
): SourceCodeLocation[] {
  return opcodeLocations.flatMap(opcodeLocation => getSourceCodeLocationsFromOpcodeLocation(opcodeLocation, debug));
}

/**
 * The function call that executes an ACIR.
 */
export async function acvm(
  solver: WasmBlackBoxFunctionSolver,
  acir: Buffer,
  initialWitness: ACVMWitness,
  callback: ACIRCallback,
): Promise<ACIRExecutionResult> {
  const logger = createDebugLogger('aztec:simulator:acvm');

  const partialWitness = await executeCircuitWithBlackBoxSolver(
    solver,
    acir,
    initialWitness,
    async (name: string, args: ForeignCallInput[]) => {
      try {
        logger(`Oracle callback ${name}`);
        const oracleFunction = callback[name as ORACLE_NAMES];
        if (!oracleFunction) {
          throw new Error(`Oracle callback ${name} not found`);
        }

        const result = await oracleFunction.call(callback, ...args);
        return [result];
      } catch (err) {
        let typedError: Error;
        if (err instanceof Error) {
          typedError = err;
        } else {
          typedError = new Error(`Error in oracle callback ${err}`);
        }
        logger.error(`Error in oracle callback ${name}`);
        throw typedError;
      }
    },
  ).catch((err: Error) => {
    // Wasm callbacks act as a boundary for stack traces, so we capture it here and complete the error if it happens.
    const stack = new Error().stack;

    traverseCauseChain(err, cause => {
      if (cause.stack) {
        cause.stack += stack;
      }
    });

    throw err;
  });

  return { partialWitness };
}

/**
 * Extracts the call stack from an thrown by the acvm.
 * @param error - The error to extract from.
 * @param debug - The debug metadata of the function called.
 * @returns The call stack, if available.
 */
export function extractCallStack(
  error: Error | ExecutionError,
  debug?: FunctionDebugMetadata,
): NoirCallStack | undefined {
  if (!('callStack' in error) || !error.callStack) {
    return undefined;
  }
  const { callStack } = error;
  if (!debug) {
    return callStack;
  }

  return resolveOpcodeLocations(callStack, debug);
}

/**
 * Adapts the buffer to the field size.
 * @param originalBuf - The buffer to adapt.
 * @returns The adapted buffer.
 */
function adaptBufferSize(originalBuf: Buffer) {
  const buffer = Buffer.alloc(Fr.SIZE_IN_BYTES);
  if (originalBuf.length > buffer.length) {
    throw new Error('Buffer does not fit in field');
  }
  originalBuf.copy(buffer, buffer.length - originalBuf.length);
  return buffer;
}

/**
 * Converts a value to an ACVM field.
 * @param value - The value to convert.
 * @returns The ACVM field.
 */
export function toACVMField(value: AztecAddress | EthAddress | Fr | Buffer | boolean | number | bigint): ACVMField {
  if (typeof value === 'boolean') {
    return value ? ONE_ACVM_FIELD : ZERO_ACVM_FIELD;
  }

  let buffer;

  if (Buffer.isBuffer(value)) {
    buffer = value;
  } else if (typeof value === 'number') {
    buffer = Buffer.alloc(Fr.SIZE_IN_BYTES);
    buffer.writeUInt32BE(value, Fr.SIZE_IN_BYTES - 4);
  } else if (typeof value === 'bigint') {
    buffer = new Fr(value).toBuffer();
  } else {
    buffer = value.toBuffer();
  }

  return `0x${adaptBufferSize(buffer).toString('hex')}`;
}

/**
 * Converts an ACVM field to a Buffer.
 * @param field - The ACVM field to convert.
 * @returns The Buffer.
 */
export function convertACVMFieldToBuffer(field: ACVMField): Buffer {
  return Buffer.from(field.slice(2), 'hex');
}

/**
 * Converts an ACVM field to a Fr.
 * @param field - The ACVM field to convert.
 * @returns The Fr.
 */
export function fromACVMField(field: ACVMField): Fr {
  return Fr.fromBuffer(convertACVMFieldToBuffer(field));
}
