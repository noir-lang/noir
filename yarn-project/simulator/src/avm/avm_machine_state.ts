import { type Fr } from '@aztec/circuits.js';

import { type Gas, GasDimensions } from './avm_gas.js';
import { TaggedMemory } from './avm_memory_types.js';
import { AvmContractCallResults } from './avm_message_call_result.js';
import { OutOfGasError } from './errors.js';

/**
 * A few fields of machine state are initialized from AVM session inputs or call instruction arguments
 */
export type InitialAvmMachineState = {
  l1GasLeft: number;
  l2GasLeft: number;
  daGasLeft: number;
};

/**
 * Avm state modified on an instruction-per-instruction basis.
 */
export class AvmMachineState {
  public l1GasLeft: number;
  /** gas remaining of the gas allocated for a contract call */
  public l2GasLeft: number;
  public daGasLeft: number;
  /** program counter */
  public pc: number = 0;

  /**
   * On INTERNALCALL, internal call stack is pushed to with the current pc + 1
   * On INTERNALRETURN, value is popped from the internal call stack and assigned to the pc.
   */
  public internalCallStack: number[] = [];

  /** Memory accessible to user code */
  public readonly memory: TaggedMemory = new TaggedMemory();

  /**
   * Signals that execution should end.
   * AvmContext execution continues executing instructions until the machine state signals "halted"
   */
  public halted: boolean = false;
  /** Signals that execution has reverted normally (this does not cover exceptional halts) */
  private reverted: boolean = false;
  /** Output data must NOT be modified once it is set */
  private output: Fr[] = [];

  constructor(l1GasLeft: number, l2GasLeft: number, daGasLeft: number) {
    this.l1GasLeft = l1GasLeft;
    this.l2GasLeft = l2GasLeft;
    this.daGasLeft = daGasLeft;
  }

  public static fromState(state: InitialAvmMachineState): AvmMachineState {
    return new AvmMachineState(state.l1GasLeft, state.l2GasLeft, state.daGasLeft);
  }

  /**
   * Consumes the given gas.
   * Should any of the gas dimensions get depleted, it sets all gas left to zero and triggers
   * an exceptional halt by throwing an OutOfGasError.
   */
  public consumeGas(gasCost: Partial<Gas>) {
    // Assert there is enough gas on every dimension.
    const outOfGasDimensions = GasDimensions.filter(
      dimension => this[`${dimension}Left`] - (gasCost[dimension] ?? 0) < 0,
    );
    // If not, trigger an exceptional halt.
    // See https://yp-aztec.netlify.app/docs/public-vm/execution#gas-checks-and-tracking
    if (outOfGasDimensions.length > 0) {
      this.exceptionalHalt();
      throw new OutOfGasError(outOfGasDimensions);
    }
    // Otherwise, charge the corresponding gas
    for (const dimension of GasDimensions) {
      this[`${dimension}Left`] -= gasCost[dimension] ?? 0;
    }
  }

  /** Increases the gas left by the amounts specified. */
  public refundGas(gasRefund: Partial<Gas>) {
    for (const dimension of GasDimensions) {
      this[`${dimension}Left`] += gasRefund[dimension] ?? 0;
    }
  }

  /**
   * Most instructions just increment PC before they complete
   */
  public incrementPc() {
    this.pc++;
  }

  /**
   * Halt as successful
   * Output data must NOT be modified once it is set
   * @param output
   */
  public return(output: Fr[]) {
    this.halted = true;
    this.output = output;
  }

  /**
   * Halt as reverted
   * Output data must NOT be modified once it is set
   * @param output
   */
  public revert(output: Fr[]) {
    this.halted = true;
    this.reverted = true;
    this.output = output;
  }

  /**
   * Flag an exceptional halt. Clears gas left and sets the reverted flag. No output data.
   */
  protected exceptionalHalt() {
    GasDimensions.forEach(dimension => (this[`${dimension}Left`] = 0));
    this.reverted = true;
    this.halted = true;
  }

  /**
   * Get a summary of execution results for a halted machine state
   * @returns summary of execution results
   */
  public getResults(): AvmContractCallResults {
    if (!this.halted) {
      throw new Error('Execution results are not ready! Execution is ongoing.');
    }
    return new AvmContractCallResults(this.reverted, this.output);
  }
}
