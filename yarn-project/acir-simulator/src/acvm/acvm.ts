import { FunctionDebugMetadata } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import {
  ForeignCallInput,
  ForeignCallOutput,
  WasmBlackBoxFunctionSolver,
  WitnessMap,
  executeCircuitWithBlackBoxSolver,
} from 'acvm_js';

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
  | 'createCommitment'
  | 'createL2ToL1Message'
  | 'createNullifier'
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
 * Extracts the opcode location from an ACVM error string.
 */
function extractOpcodeLocationFromError(err: string): string | undefined {
  const match = err.match(/^Cannot satisfy constraint (?<opcodeLocation>[0-9]+(?:\.[0-9]+)?)/);
  return match?.groups?.opcodeLocation;
}

/**
 * The data for a call in the call stack.
 */
interface SourceCodeLocation {
  /**
   * The path to the source file.
   */
  filePath: string;
  /**
   * The line number of the call.
   */
  line: number;
  /**
   * The source code of the file.
   */
  fileSource: string;
  /**
   * The source code text of the failed constraint.
   */
  assertionText: string;
}

/**
 * Extracts the call stack from the location of a failing opcode and the debug metadata.
 */
function getCallStackFromOpcodeLocation(opcodeLocation: string, debug: FunctionDebugMetadata): SourceCodeLocation[] {
  const { debugSymbols, files } = debug;

  const callStack = debugSymbols.locations[opcodeLocation] || [];
  return callStack.map(call => {
    const { file: fileId, span } = call;

    const { path, source } = files[fileId];

    const assertionText = source.substring(span.start, span.end + 1);
    const precedingText = source.substring(0, span.start);
    const line = precedingText.split('\n').length;

    return {
      filePath: path,
      line,
      fileSource: source,
      assertionText,
    };
  });
}

/**
 * Creates a formatted string for an error stack
 * @param callStack - The error stack
 * @returns - The formatted string
 */
function printErrorStack(callStack: SourceCodeLocation[]): string {
  // TODO experiment with formats of reporting this for better error reporting
  return [
    'Error: Assertion failed',
    callStack.map(call => `  at ${call.filePath}:${call.line} '${call.assertionText}'`),
  ].join('\n');
}

/**
 * The function call that executes an ACIR.
 */
export async function acvm(
  solver: WasmBlackBoxFunctionSolver,
  acir: Buffer,
  initialWitness: ACVMWitness,
  callback: ACIRCallback,
  debug?: FunctionDebugMetadata,
): Promise<ACIRExecutionResult> {
  const logger = createDebugLogger('aztec:simulator:acvm');
  // This is a workaround to avoid the ACVM removing the information about the underlying error.
  // We should probably update the ACVM to let proper errors through.
  let oracleError: Error | undefined = undefined;

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
        oracleError = typedError;
        logger.error(`Error in oracle callback ${name}: ${typedError.message}`);
        throw typedError;
      }
    },
  ).catch((acvmError: string) => {
    if (oracleError) {
      throw oracleError;
    }
    const opcodeLocation = extractOpcodeLocationFromError(acvmError);
    if (!opcodeLocation || !debug) {
      throw new Error(acvmError);
    }

    const callStack = getCallStackFromOpcodeLocation(opcodeLocation, debug);
    logger(printErrorStack(callStack));
    throw new Error(`Assertion failed: '${callStack.pop()?.assertionText ?? 'Unknown'}'`);
  });

  return Promise.resolve({ partialWitness });
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
