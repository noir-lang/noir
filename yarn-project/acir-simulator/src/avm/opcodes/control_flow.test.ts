import { MockProxy, mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { TypeTag, Uint16 } from '../avm_memory_types.js';
import { initExecutionEnvironment } from '../fixtures/index.js';
import { AvmJournal } from '../journal/journal.js';
import { Add, Mul, Sub } from './arithmetic.js';
import { And, Not, Or, Shl, Shr, Xor } from './bitwise.js';
import { Eq, Lt, Lte } from './comparators.js';
import { InternalCall, InternalReturn, Jump, JumpI } from './control_flow.js';
import { InstructionExecutionError } from './instruction.js';
import { CMov, CalldataCopy, Cast, Mov, Set } from './memory.js';

describe('Control Flow Opcodes', () => {
  let journal: MockProxy<AvmJournal>;
  let machineState: AvmMachineState;

  beforeEach(() => {
    journal = mock<AvmJournal>();
    machineState = new AvmMachineState(initExecutionEnvironment());
  });

  it('Should implement JUMP', async () => {
    const jumpLocation = 22;

    expect(machineState.pc).toBe(0);

    const instruction = new Jump(jumpLocation);
    await instruction.execute(machineState, journal);
    expect(machineState.pc).toBe(jumpLocation);
  });

  it('Should implement JUMPI - truthy', async () => {
    const jumpLocation = 22;
    const jumpLocation1 = 69;

    expect(machineState.pc).toBe(0);

    machineState.memory.set(0, new Uint16(1n));
    machineState.memory.set(1, new Uint16(2n));

    const instruction = new JumpI(jumpLocation, 0);
    await instruction.execute(machineState, journal);
    expect(machineState.pc).toBe(jumpLocation);

    // Truthy can be greater than 1
    const instruction1 = new JumpI(jumpLocation1, 1);
    await instruction1.execute(machineState, journal);
    expect(machineState.pc).toBe(jumpLocation1);
  });

  it('Should implement JUMPI - falsy', async () => {
    const jumpLocation = 22;

    expect(machineState.pc).toBe(0);

    machineState.memory.set(0, new Uint16(0n));

    const instruction = new JumpI(jumpLocation, 0);
    await instruction.execute(machineState, journal);
    expect(machineState.pc).toBe(1);
  });

  it('Should implement Internal Call and Return', async () => {
    const jumpLocation = 22;

    expect(machineState.pc).toBe(0);

    const instruction = new InternalCall(jumpLocation);
    const returnInstruction = new InternalReturn();

    await instruction.execute(machineState, journal);
    expect(machineState.pc).toBe(jumpLocation);

    await returnInstruction.execute(machineState, journal);
    expect(machineState.pc).toBe(1);
  });

  it('Should chain series of control flow instructions', async () => {
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
      await instructions[i].execute(machineState, journal);
      expect(machineState.pc).toBe(expectedPcs[i]);
    }
  });

  it('Should error if Internal Return is called without a corresponding Internal Call', async () => {
    const returnInstruction = () => new InternalReturn().execute(machineState, journal);
    await expect(returnInstruction()).rejects.toThrow(InstructionExecutionError);
  });

  it('Should increment PC on All other Instructions', async () => {
    const instructions = [
      new Add(0, 1, 2),
      new Sub(0, 1, 2),
      new Mul(0, 1, 2),
      new Lt(TypeTag.UINT16, 0, 1, 2),
      new Lte(TypeTag.UINT16, 0, 1, 2),
      new Eq(TypeTag.UINT16, 0, 1, 2),
      new Xor(TypeTag.UINT16, 0, 1, 2),
      new And(TypeTag.UINT16, 0, 1, 2),
      new Or(TypeTag.UINT16, 0, 1, 2),
      new Shl(TypeTag.UINT16, 0, 1, 2),
      new Shr(TypeTag.UINT16, 0, 1, 2),
      new Not(TypeTag.UINT16, 0, 2),
      new CalldataCopy(0, 1, 2),
      new Set(TypeTag.UINT16, 0n, 1),
      new Mov(0, 1),
      new CMov(0, 1, 2, 3),
      new Cast(TypeTag.UINT16, 0, 1),
    ];

    for (const instruction of instructions) {
      // Use a fresh machine state each run
      const innerMachineState = new AvmMachineState(initExecutionEnvironment());
      innerMachineState.memory.set(0, new Uint16(4n));
      innerMachineState.memory.set(1, new Uint16(8n));
      innerMachineState.memory.set(2, new Uint16(12n));
      expect(machineState.pc).toBe(0);
      await instruction.execute(innerMachineState, journal);
      expect(innerMachineState.pc).toBe(1);
    }
  });
});
