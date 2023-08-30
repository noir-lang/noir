import { AztecAddress, FunctionSelector } from '@aztec/circuits.js';

/**
 * Address and selector of a function that failed during simulation.
 */
export interface FailingFunction {
  /**
   * The address of the contract that failed.
   */
  contractAddress: AztecAddress;
  /**
   * The name of the contract that failed.
   */
  contractName?: string;
  /**
   * The selector of the function that failed.
   */
  functionSelector: FunctionSelector;
  /**
   * The name of the function that failed.
   */
  functionName?: string;
}

/**
 * A pointer to a failing section of the noir source code.
 */
export interface SourceCodeLocation {
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
  locationText: string;
}

/**
 * A stack of noir source code locations.
 */
export type NoirCallStack = SourceCodeLocation[];

/**
 * An error during the simulation of a function call.
 */
export class SimulationError extends Error {
  private functionErrorStack: FailingFunction[];

  // We want to maintain a public constructor for proper printing.
  constructor(
    message: string,
    failingFunction: FailingFunction,
    private noirErrorStack?: NoirCallStack,
    options?: ErrorOptions,
  ) {
    super(message, options);
    this.functionErrorStack = [failingFunction];
  }

  private addCaller(failingFunction: FailingFunction) {
    this.functionErrorStack.unshift(failingFunction);
  }

  static fromError(
    failingContract: AztecAddress,
    failingselector: FunctionSelector,
    err: Error & {
      /**
       * The noir call stack.
       */
      callStack?: NoirCallStack;
    },
  ) {
    const failingFunction = { contractAddress: failingContract, functionSelector: failingselector };
    if (err instanceof SimulationError) {
      return SimulationError.extendPreviousSimulationError(failingFunction, err);
    }
    return new SimulationError(err.message, failingFunction, err?.callStack, {
      cause: err,
    });
  }

  static extendPreviousSimulationError(failingFunction: FailingFunction, previousError: SimulationError) {
    previousError.addCaller(failingFunction);
    return previousError;
  }

  /**
   * Enriches the error with the name of a contract that failed.
   * @param contractAddress - The address of the contract
   * @param contractName - The corresponding name
   */
  enrichWithContractName(contractAddress: AztecAddress, contractName: string) {
    this.functionErrorStack.forEach(failingFunction => {
      if (failingFunction.contractAddress.equals(contractAddress)) {
        failingFunction.contractName = contractName;
      }
    });
  }

  /**
   * Enriches the error with the name of a function that failed.
   * @param contractAddress - The address of the contract
   * @param functionSelector - The selector of the function
   * @param functionName - The corresponding name
   */
  enrichWithFunctionName(contractAddress: AztecAddress, functionSelector: FunctionSelector, functionName: string) {
    this.functionErrorStack.forEach(failingFunction => {
      if (
        failingFunction.contractAddress.equals(contractAddress) &&
        failingFunction.functionSelector.equals(functionSelector)
      ) {
        failingFunction.functionName = functionName;
      }
    });
  }

  /**
   * Returns a string representation of the error.
   * @returns The string.
   */
  toString() {
    const functionCallStack = this.getCallStack();
    const noirCallStack = this.getNoirCallStack();

    // Try to resolve the contract and function names of the stack of failing functions.
    const stackLines: string[] = [
      ...functionCallStack.map(failingFunction => {
        return `  at ${failingFunction.contractName ?? failingFunction.contractAddress.toString()}.${
          failingFunction.functionName ?? failingFunction.functionSelector.toString()
        }`;
      }),
      ...noirCallStack.map(
        sourceCodeLocation =>
          `  at ${sourceCodeLocation.filePath}:${sourceCodeLocation.line} '${sourceCodeLocation.locationText}'`,
      ),
    ];

    return [`Simulation error: ${this.message}`, ...stackLines.reverse()].join('\n');
  }

  /**
   * Updates the error message. This is needed because in some engines the stack also contains the message.
   * @param newMessage - The new message of this error.
   */
  updateMessage(newMessage: string) {
    const oldMessage = this.message;
    this.message = newMessage;
    if (this.stack?.startsWith(`Error: ${oldMessage}`)) {
      this.stack = this.stack?.replace(`Error: ${oldMessage}`, `Error: ${newMessage}`);
    }
  }

  /**
   * The aztec function stack that failed during simulation.
   */
  getCallStack(): FailingFunction[] {
    return this.functionErrorStack;
  }

  /**
   * Returns the noir call stack inside the first function that failed during simulation.
   * @returns The noir call stack.
   */
  getNoirCallStack(): NoirCallStack {
    return this.noirErrorStack || [];
  }

  /**
   * Sets the noir call stack.
   * @param callStack - The noir call stack.
   */
  setNoirCallStack(callStack: NoirCallStack) {
    this.noirErrorStack = callStack;
  }

  toJSON() {
    return {
      message: this.message,
      functionErrorStack: this.functionErrorStack,
      noirErrorStack: this.noirErrorStack,
    };
  }

  static fromJSON(obj: any) {
    const error = new SimulationError(obj.message, obj.functionErrorStack[0], obj.noirErrorStack);
    error.functionErrorStack = obj.functionErrorStack;
    return error;
  }
}
