import { Fr } from '@aztec/foundation/fields';

import { mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Add, Mul, Sub } from './arithmetic.js';
import { And, Not, Or, Shl, Shr, Xor } from './bitwise.js';
import { Eq, Lt, Lte } from './comparators.js';
import { InternalCall, InternalCallStackEmptyError, InternalReturn, Jump, JumpI } from './control_flow.js';
import { CMov, CalldataCopy, Cast, Mov, Set } from './memory.js';

describe('Control Flow Opcodes', () => {
  let stateManager = mock<AvmStateManager>();
  let machineState: AvmMachineState;

  beforeEach(() => {
    stateManager = mock<AvmStateManager>();
    machineState = new AvmMachineState([]);
  });

  it('Should implement JUMP', () => {
    const jumpLocation = 22;

    expect(machineState.pc).toBe(0);

    const instruction = new Jump(jumpLocation);
    instruction.execute(machineState, stateManager);
    expect(machineState.pc).toBe(jumpLocation);
  });

  it('Should implement JUMPI - truthy', () => {
    const jumpLocation = 22;
    const jumpLocation1 = 69;

    expect(machineState.pc).toBe(0);

    machineState.writeMemory(0, new Fr(1n));
    machineState.writeMemory(1, new Fr(2n));

    const instruction = new JumpI(jumpLocation, 0);
    instruction.execute(machineState, stateManager);
    expect(machineState.pc).toBe(jumpLocation);

    // Truthy can be greater than 1
    const instruction1 = new JumpI(jumpLocation1, 1);
    instruction1.execute(machineState, stateManager);
    expect(machineState.pc).toBe(jumpLocation1);
  });

  it('Should implement JUMPI - falsy', () => {
    const jumpLocation = 22;

    expect(machineState.pc).toBe(0);

    machineState.writeMemory(0, new Fr(0n));

    const instruction = new JumpI(jumpLocation, 0);
    instruction.execute(machineState, stateManager);
    expect(machineState.pc).toBe(1);
  });

  it('Should implement Internal Call and Return', () => {
    const jumpLocation = 22;

    expect(machineState.pc).toBe(0);

    const instruction = new InternalCall(jumpLocation);
    const returnInstruction = new InternalReturn();

    instruction.execute(machineState, stateManager);
    expect(machineState.pc).toBe(jumpLocation);

    returnInstruction.execute(machineState, stateManager);
    expect(machineState.pc).toBe(1);
  });

  it('Should chain series of control flow instructions', () => {
    const jumpLocation0 = 22;
    const jumpLocation1 = 69;
    const jumpLocation2 = 1337;

    const aloneJumpLocation = 420;

    const instructions = [
      // pc  |  internal call stack
      new InternalCall(jumpLocation0), // 22  | [1]
      new InternalCall(jumpLocation1), // 69  | [1, 23]
      new InternalReturn(), // 23  | [1]
      new Jump(aloneJumpLocation), // 420 | [1]
      new InternalCall(jumpLocation2), // 1337| [1, 421]
      new InternalReturn(), // 421 | [1]
      new InternalReturn(), // 1   | []
    ];

    // The expected program counter after each instruction is invoked
    const expectedPcs = [
      jumpLocation0,
      jumpLocation1,
      jumpLocation0 + 1,
      aloneJumpLocation,
      jumpLocation2,
      aloneJumpLocation + 1,
      1,
    ];

    for (let i = 0; i < instructions.length; i++) {
      instructions[i].execute(machineState, stateManager);
      expect(machineState.pc).toBe(expectedPcs[i]);
    }
  });

  it('Should error if Internal Return is called without a corresponding Internal Call', () => {
    const returnInstruction = new InternalReturn();
    expect(() => returnInstruction.execute(machineState, stateManager)).toThrow(InternalCallStackEmptyError);
  });

  it('Should increment PC on All other Instructions', () => {
    const instructions = [
      new Add(0, 1, 2),
      new Sub(0, 1, 2),
      new Mul(0, 1, 2),
      new Lt(0, 1, 2),
      new Lte(0, 1, 2),
      new Eq(0, 1, 2),
      new Xor(0, 1, 2),
      new And(0, 1, 2),
      new Or(0, 1, 2),
      new Shl(0, 1, 2),
      new Shr(0, 1, 2),
      new Not(0, 2),
      new CalldataCopy(0, 1, 2),
      new Set(0n, 1),
      new Mov(0, 1),
      new CMov(0, 1, 2, 3),
      new Cast(0, 1),
    ];

    for (const instruction of instructions) {
      // Use a fresh machine state each run
      const innerMachineState = new AvmMachineState([]);
      expect(machineState.pc).toBe(0);
      instruction.execute(innerMachineState, stateManager);
      expect(innerMachineState.pc).toBe(1);
    }
  });
});
