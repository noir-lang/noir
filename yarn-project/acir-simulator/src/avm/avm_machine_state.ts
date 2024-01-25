import { Fr } from '@aztec/foundation/fields';

import { AvmExecutionEnvironment } from './avm_execution_environment.js';
import { TaggedMemory } from './avm_memory_types.js';

/**
 * Store's data for an Avm execution frame
 */
export class AvmMachineState {
  /**
   * Execution environment contains hard coded information that is received from the kernel
   * Items like, the block header and global variables fall within this category
   */
  public readonly executionEnvironment: AvmExecutionEnvironment;

  private returnData: Fr[];

  /** - */
  public readonly memory: TaggedMemory;

  /**
   * When an internal_call is invoked, the internal call stack is added to with the current pc + 1
   * When internal_return is invoked, the latest value is popped from the internal call stack and set to the pc.
   */
  public internalCallStack: number[];

  /** - */
  public pc: number;
  /** - */
  public callStack: number[];

  /**
   * If an instruction triggers a halt, then it ends execution of the VM
   */
  public halted: boolean;

  /**
   * Create a new avm context
   * @param executionEnvironment - Machine context that is passed to the avm
   */
  constructor(executionEnvironment: AvmExecutionEnvironment) {
    this.returnData = [];
    this.memory = new TaggedMemory();
    this.internalCallStack = [];

    this.pc = 0;
    this.callStack = [];

    this.halted = false;

    this.executionEnvironment = executionEnvironment;
  }

  /**
   * Return data must NOT be modified once it is set
   * @param returnData -
   */
  public setReturnData(returnData: Fr[]) {
    this.returnData = returnData;
    Object.freeze(returnData);
  }

  /** - */
  public getReturnData(): Fr[] {
    return this.returnData;
  }
}
