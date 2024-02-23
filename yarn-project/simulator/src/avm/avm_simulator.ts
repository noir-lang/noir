import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { strict as assert } from 'assert';

import type { AvmContext } from './avm_context.js';
import { AvmContractCallResults } from './avm_message_call_result.js';
import { AvmExecutionError, InvalidProgramCounterError, NoBytecodeForContractError } from './errors.js';
import type { Instruction } from './opcodes/index.js';
import { decodeFromBytecode } from './serialization/bytecode_serialization.js';

export class AvmSimulator {
  private log: DebugLogger = createDebugLogger('aztec:avm_simulator');

  constructor(private context: AvmContext) {}

  /**
   * Fetch the bytecode and execute it in the current context.
   */
  public async execute(): Promise<AvmContractCallResults> {
    const instructions = await this.fetchAndDecodeBytecode();
    return this.executeInstructions(instructions);
  }

  /**
   * Executes the provided instructions in the current context.
   * This method is useful for testing and debugging.
   */
  public async executeInstructions(instructions: Instruction[]): Promise<AvmContractCallResults> {
    assert(instructions.length > 0);

    try {
      // Execute instruction pointed to by the current program counter
      // continuing until the machine state signifies a halt
      while (!this.context.machineState.halted) {
        const instruction = instructions[this.context.machineState.pc];
        assert(!!instruction); // This should never happen

        this.log(`Executing PC=${this.context.machineState.pc}: ${instruction.toString()}`);
        // Execute the instruction.
        // Normal returns and reverts will return normally here.
        // "Exceptional halts" will throw.
        await instruction.execute(this.context);

        if (this.context.machineState.pc >= instructions.length) {
          this.log('Passed end of program!');
          throw new InvalidProgramCounterError(this.context.machineState.pc, /*max=*/ instructions.length);
        }
      }

      // Return results for processing by calling context
      const results = this.context.machineState.getResults();
      this.log(`Context execution results: ${results.toString()}`);
      return results;
    } catch (e) {
      this.log('Exceptional halt');
      if (!(e instanceof AvmExecutionError)) {
        this.log(`Unknown error thrown by avm: ${e}`);
        throw e;
      }

      // Return results for processing by calling context
      // Note: "exceptional halts" cannot return data
      const results = new AvmContractCallResults(/*reverted=*/ true, /*output=*/ [], /*revertReason=*/ e);
      this.log(`Context execution results: ${results.toString()}`);
      return results;
    }
  }

  /**
   * Fetch contract bytecode from world state and decode into executable instructions.
   */
  private async fetchAndDecodeBytecode(): Promise<Instruction[]> {
    // NOTE: the following is mocked as getPublicBytecode does not exist yet

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

    return decodeFromBytecode(bytecode);
  }
}
