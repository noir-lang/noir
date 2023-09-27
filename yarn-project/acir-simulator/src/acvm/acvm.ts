import { FunctionDebugMetadata, OpcodeLocation } from '@aztec/foundation/abi';
import { createDebugLogger } from '@aztec/foundation/log';
import { NoirCallStack, SourceCodeLocation } from '@aztec/types';

import {
  ExecutionError,
  ForeignCallInput,
  ForeignCallOutput,
  WasmBlackBoxFunctionSolver,
  WitnessMap,
  executeCircuitWithBlackBoxSolver,
} from '@noir-lang/acvm_js';

import { traverseCauseChain } from '../common/errors.js';
import { ORACLE_NAMES } from './oracle/index.js';

/**
 * The format for fields on the ACVM.
 */
export type ACVMField = string;

/**
 * The format for witnesses of the ACVM.
 */
export type ACVMWitness = WitnessMap;

/**
 * The callback interface for the ACIR.
 */
type ACIRCallback = Record<
  ORACLE_NAMES,
  (...args: ForeignCallInput[]) => ForeignCallOutput | Promise<ForeignCallOutput>
>;

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

    const locationText = source.substring(span.start, span.end);
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
