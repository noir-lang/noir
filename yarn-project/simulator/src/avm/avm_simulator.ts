import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { strict as assert } from 'assert';

import type { AvmContext } from './avm_context.js';
import { AvmContractCallResults } from './avm_message_call_result.js';
import { AvmExecutionError, InvalidProgramCounterError, NoBytecodeForContractError } from './errors.js';
import type { Instruction } from './opcodes/index.js';
import { decodeFromBytecode } from './serialization/bytecode_serialization.js';

export class AvmSimulator {
  private log: DebugLogger;

  constructor(private context: AvmContext) {
    this.log = createDebugLogger(
      `aztec:avm_simulator:core(f:${context.environment.temporaryFunctionSelector.toString()})`,
    );
  }

  /**
   * Fetch the bytecode and execute it in the current context.
   */
  public async execute(): Promise<AvmContractCallResults> {
    const selector = this.context.environment.temporaryFunctionSelector;
    const bytecode = await this.context.persistableState.hostStorage.contractsDb.getBytecode(
      this.context.environment.address,
      selector,
    );

    // This assumes that we will not be able to send messages to accounts without code
    // Pending classes and instances impl details
    if (!bytecode) {
      throw new NoBytecodeForContractError(this.context.environment.address);
    }

    return await this.executeBytecode(bytecode);
  }

  /**
   * Executes the provided bytecode in the current context.
   * This method is useful for testing and debugging.
   */
  public async executeBytecode(bytecode: Buffer): Promise<AvmContractCallResults> {
    return await this.executeInstructions(decodeFromBytecode(bytecode));
  }

  /**
   * Executes the provided instructions in the current context.
   * This method is useful for testing and debugging.
   */
  public async executeInstructions(instructions: Instruction[]): Promise<AvmContractCallResults> {
    assert(instructions.length > 0);
    const { machineState } = this.context;
    try {
      // Execute instruction pointed to by the current program counter
      // continuing until the machine state signifies a halt
      while (!machineState.halted) {
        const instruction = instructions[machineState.pc];
        assert(
          !!instruction,
          'AVM attempted to execute non-existent instruction. This should never happen (invalid bytecode or AVM simulator bug)!',
        );

        const gasLeft = `l1=${machineState.l1GasLeft} l2=${machineState.l2GasLeft} da=${machineState.daGasLeft}`;
        this.log.debug(`@${machineState.pc} ${instruction.toString()} (${gasLeft})`);
        // Execute the instruction.
        // Normal returns and reverts will return normally here.
        // "Exceptional halts" will throw.
        await instruction.execute(this.context);

        if (machineState.pc >= instructions.length) {
          this.log.warn('Passed end of program');
          throw new InvalidProgramCounterError(machineState.pc, /*max=*/ instructions.length);
        }
      }

      // Return results for processing by calling context
      const results = machineState.getResults();
      this.log.debug(`Context execution results: ${results.toString()}`);
      return results;
    } catch (e) {
      this.log.verbose('Exceptional halt');
      if (!(e instanceof AvmExecutionError)) {
        this.log.verbose(`Unknown error thrown by avm: ${e}`);
        throw e;
      }

      // Return results for processing by calling context
      // Note: "exceptional halts" cannot return data
      const results = new AvmContractCallResults(/*reverted=*/ true, /*output=*/ [], /*revertReason=*/ e);
      this.log.debug(`Context execution results: ${results.toString()}`);
      return results;
    }
  }
}
