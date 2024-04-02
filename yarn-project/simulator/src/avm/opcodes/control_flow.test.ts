import { type AvmContext } from '../avm_context.js';
import { Uint16 } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { initContext } from '../fixtures/index.js';
import { InternalCall, InternalReturn, Jump, JumpI } from './control_flow.js';

describe('Control Flow Opcodes', () => {
  let context: AvmContext;

  beforeEach(() => {
    context = initContext();
  });

  describe('JUMP', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Jump.opcode, // opcode
        ...Buffer.from('12345678', 'hex'), // loc
      ]);
      const inst = new Jump(/*loc=*/ 0x12345678);

      expect(Jump.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should implement JUMP', async () => {
      const jumpLocation = 22;

      expect(context.machineState.pc).toBe(0);

      const instruction = new Jump(jumpLocation);
      await instruction.execute(context);
      expect(context.machineState.pc).toBe(jumpLocation);
    });
  });

  describe('JUMPI', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        JumpI.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // loc
        ...Buffer.from('a2345678', 'hex'), // condOffset
      ]);
      const inst = new JumpI(/*indirect=*/ 1, /*loc=*/ 0x12345678, /*condOffset=*/ 0xa2345678);

      expect(JumpI.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should implement JUMPI - truthy', async () => {
      const jumpLocation = 22;
      const jumpLocation1 = 69;

      expect(context.machineState.pc).toBe(0);

      context.machineState.memory.set(0, new Uint16(1n));
      context.machineState.memory.set(1, new Uint16(2n));

      const instruction = new JumpI(/*indirect=*/ 0, jumpLocation, /*condOffset=*/ 0);
      await instruction.execute(context);
      expect(context.machineState.pc).toBe(jumpLocation);

      // Truthy can be greater than 1
      const instruction1 = new JumpI(/*indirect=*/ 0, jumpLocation1, /*condOffset=*/ 1);
      await instruction1.execute(context);
      expect(context.machineState.pc).toBe(jumpLocation1);
    });

    it('Should implement JUMPI - falsy', async () => {
      const jumpLocation = 22;

      expect(context.machineState.pc).toBe(0);

      context.machineState.memory.set(0, new Uint16(0n));

      const instruction = new JumpI(/*indirect=*/ 0, jumpLocation, /*condOffset=*/ 0);
      await instruction.execute(context);
      expect(context.machineState.pc).toBe(1);
    });
  });

  describe('INTERNALCALL and INTERNALRETURN', () => {
    it('INTERNALCALL Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        InternalCall.opcode, // opcode
        ...Buffer.from('12345678', 'hex'), // loc
      ]);
      const inst = new InternalCall(/*loc=*/ 0x12345678);

      expect(InternalCall.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should implement Internal Call and Return', async () => {
      const jumpLocation = 22;

      expect(context.machineState.pc).toBe(0);

      const instruction = new InternalCall(jumpLocation);
      const returnInstruction = new InternalReturn();

      await instruction.execute(context);
      expect(context.machineState.pc).toBe(jumpLocation);

      await returnInstruction.execute(context);
      expect(context.machineState.pc).toBe(1);
    });

    it('Should error if Internal Return is called without a corresponding Internal Call', async () => {
      const returnInstruction = () => new InternalReturn().execute(context);
      await expect(returnInstruction()).rejects.toThrow(InstructionExecutionError);
    });
  });

  describe('General flow', () => {
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
        await instructions[i].execute(context);
        expect(context.machineState.pc).toBe(expectedPcs[i]);
      }
    });
  });
});
