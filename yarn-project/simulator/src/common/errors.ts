import { type FailingFunction, type NoirCallStack, SimulationError } from '@aztec/circuit-types';

/**
 * An error that occurred during the execution of a function.
 * @param message - the error message
 * @param failingFunction - the Aztec function that failed
 * @param noirCallStack - the internal call stack of the function that failed (within the failing Aztec function)
 * @param options - additional error options (an optional "cause" entry allows for a recursive error stack where
 *                  an error's cause may be an ExecutionError itself)
 */
export class ExecutionError extends Error {
  constructor(
    message: string,
    /**
     * The function that failed.
     */
    public failingFunction: FailingFunction,
    /**
     * The noir call stack of the function that failed.
     */
    public noirCallStack?: NoirCallStack,
    options?: ErrorOptions,
  ) {
    super(message, options);
  }
}

/**
 * Traverses the cause chain of an error.
 * @param error - The error to start from.
 * @param callback - A callback on every error, including the first one.
 */
export function traverseCauseChain(error: Error, callback: (error: Error) => void) {
  let currentError: Error | undefined = error;
  while (currentError) {
    callback(currentError);
    if (currentError.cause && currentError.cause instanceof Error) {
      currentError = currentError.cause;
    } else {
      currentError = undefined;
    }
  }
}

/**
 * Creates a simulation error from an error chain generated during the execution of a function.
 * @param error - The error thrown during execution.
 * @returns - A simulation error.
 */
export function createSimulationError(error: Error): SimulationError {
  let rootCause = error;
  let noirCallStack: NoirCallStack | undefined = undefined;
  const aztecCallStack: FailingFunction[] = [];

  traverseCauseChain(error, cause => {
    rootCause = cause;
    if (cause instanceof ExecutionError) {
      aztecCallStack.push(cause.failingFunction);
      if (cause.noirCallStack) {
        noirCallStack = cause.noirCallStack;
      }
    }
  });

  return new SimulationError(rootCause.message, aztecCallStack, noirCallStack, { cause: rootCause });
}
