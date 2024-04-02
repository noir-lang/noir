import { type AztecAddress } from '@aztec/circuits.js';

/**
 * Avm-specific errors should derive from this
 */
export abstract class AvmExecutionError extends Error {
  constructor(message: string, ...rest: any[]) {
    super(message, ...rest);
    this.name = 'AvmInterpreterError';
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
