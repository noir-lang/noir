import { type FailingFunction, type NoirCallStack } from '@aztec/circuit-types';
import { type AztecAddress, type Fr } from '@aztec/circuits.js';

import { ExecutionError } from '../common/errors.js';
import { type AvmContext } from './avm_context.js';

/**
 * Avm-specific errors should derive from this
 */
export abstract class AvmExecutionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AvmExecutionError';
  }
}

export class NoBytecodeForContractError extends AvmExecutionError {
  constructor(contractAddress: AztecAddress) {
    super(`No bytecode found at: ${contractAddress}`);
    this.name = 'NoBytecodeFoundInterpreterError';
  }
}

/**
 * Error is thrown when the program counter goes to an invalid location.
 * There is no instruction at the provided pc
 */
export class InvalidProgramCounterError extends AvmExecutionError {
  constructor(pc: number, max: number) {
    super(`Invalid program counter ${pc}, max is ${max}`);
    this.name = 'InvalidProgramCounterError';
  }
}

/**
 * Error thrown during an instruction's execution (during its execute()).
 */
export class InstructionExecutionError extends AvmExecutionError {
  constructor(message: string) {
    super(message);
    this.name = 'InstructionExecutionError';
  }
}

/**
 * Error thrown on failed AVM memory tag check.
 */
export class TagCheckError extends AvmExecutionError {
  public static forOffset(offset: number, gotTag: string, expectedTag: string): TagCheckError {
    return new TagCheckError(`Tag mismatch at offset ${offset}, got ${gotTag}, expected ${expectedTag}`);
  }

  public static forTag(gotTag: string, expectedTag: string): TagCheckError {
    return new TagCheckError(`Tag mismatch, got ${gotTag}, expected ${expectedTag}`);
  }

  constructor(message: string) {
    super(message);
    this.name = 'TagCheckError';
  }
}

/** Error thrown when out of gas. */
export class OutOfGasError extends AvmExecutionError {
  constructor(dimensions: string[]) {
    super(`Not enough ${dimensions.map(d => d.toUpperCase()).join(', ')} gas left`);
    this.name = 'OutOfGasError';
  }
}

/**
 * Error is thrown when a static call attempts to alter some state
 */
export class StaticCallAlterationError extends InstructionExecutionError {
  constructor() {
    super('Static call cannot update the state, emit L2->L1 messages or generate logs');
    this.name = 'StaticCallAlterationError';
  }
}

/**
 * Error thrown to propagate a nested call's revert.
 * @param message - the error's message
 * @param nestedError - the revert reason of the nested call
 */
export class RethrownError extends AvmExecutionError {
  constructor(message: string, public nestedError: AvmRevertReason) {
    super(message);
    this.name = 'RethrownError';
  }
}

/**
 * Meaningfully named alias for ExecutionError when used in the context of the AVM.
 * Maintains a recursive structure reflecting the AVM's external callstack/errorstack, where
 * options.cause is the error that caused this error (if this is not the root-cause itself).
 */
export class AvmRevertReason extends ExecutionError {
  constructor(message: string, failingFunction: FailingFunction, noirCallStack: NoirCallStack, options?: ErrorOptions) {
    super(message, failingFunction, noirCallStack, options);
  }
}

/**
 * Helper to create a "revert reason" error optionally with a nested error cause.
 *
 * @param message - the error message
 * @param context - the context of the AVM execution used to extract the failingFunction and noirCallStack
 * @param nestedError - the error that caused this one (if this is not the root-cause itself)
 */
function createRevertReason(message: string, context: AvmContext, nestedError?: AvmRevertReason): AvmRevertReason {
  return new AvmRevertReason(
    message,
    /*failingFunction=*/ {
      contractAddress: context.environment.address,
      functionSelector: context.environment.temporaryFunctionSelector,
    },
    /*noirCallStack=*/ [...context.machineState.internalCallStack, context.machineState.pc].map(pc => `0.${pc}`),
    /*options=*/ { cause: nestedError },
  );
}

/**
 * Create a "revert reason" error for an exceptional halt,
 * creating the recursive structure if the halt was a RethrownError.
 *
 * @param haltingError - the lower-level error causing the exceptional halt
 * @param context - the context of the AVM execution used to extract the failingFunction and noirCallStack
 */
export function revertReasonFromExceptionalHalt(haltingError: AvmExecutionError, context: AvmContext): AvmRevertReason {
  // A RethrownError has a nested/child AvmRevertReason
  const nestedError = haltingError instanceof RethrownError ? haltingError.nestedError : undefined;
  return createRevertReason(haltingError.message, context, nestedError);
}

/**
 * Create a "revert reason" error for an explicit revert (a root cause).
 *
 * @param revertData - output data of the explicit REVERT instruction
 * @param context - the context of the AVM execution used to extract the failingFunction and noirCallStack
 */
export function revertReasonFromExplicitRevert(revertData: Fr[], context: AvmContext): AvmRevertReason {
  const revertMessage = decodeRevertDataAsMessage(revertData);
  return createRevertReason(revertMessage, context);
}

/**
 * Interpret revert data as a message string.
 *
 * @param revertData - output data of an explicit REVERT instruction
 */
export function decodeRevertDataAsMessage(revertData: Fr[]): string {
  if (revertData.length === 0) {
    return 'Assertion failed.';
  } else {
    try {
      // We remove the first element which is the 'error selector'.
      const revertOutput = revertData.slice(1);
      // Try to interpret the output as a text string.
      return 'Assertion failed: ' + String.fromCharCode(...revertOutput.map(fr => fr.toNumber()));
    } catch (e) {
      return 'Assertion failed: <cannot interpret as string>';
    }
  }
}
