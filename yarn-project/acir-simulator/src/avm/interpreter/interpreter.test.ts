import { Fr } from '@aztec/foundation/fields';

import { MockProxy, mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { initExecutionEnvironment } from '../fixtures/index.js';
import { AvmJournal } from '../journal/journal.js';
import { Add } from '../opcodes/arithmetic.js';
import { Jump, Return } from '../opcodes/control_flow.js';
import { Instruction } from '../opcodes/instruction.js';
import { CalldataCopy } from '../opcodes/memory.js';
import { AvmInterpreter, InvalidProgramCounterError } from './interpreter.js';

describe('interpreter', () => {
  let journal: MockProxy<AvmJournal>;

  beforeEach(() => {
    journal = mock<AvmJournal>();
  });

  it('Should execute a series of instructions', async () => {
    const calldata: Fr[] = [new Fr(1), new Fr(2)];

    const instructions: Instruction[] = [
      new CalldataCopy(/*cdOffset=*/ 0, /*copySize=*/ 2, /*dstOffset=*/ 0),
      new Add(/*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Return(/*returnOffset=*/ 2, /*copySize=*/ 1),
    ];

    const machineState = new AvmMachineState(initExecutionEnvironment({ calldata }));
    const interpreter = new AvmInterpreter(machineState, journal, instructions);
    const avmReturnData = await interpreter.run();

    expect(avmReturnData.reverted).toBe(false);
    expect(avmReturnData.revertReason).toBeUndefined();
    expect(avmReturnData.output).toEqual([new Fr(3)]);
  });

  it('Should revert with an invalid jump', async () => {
    const calldata: Fr[] = [];

    const invalidJumpDestination = 22;

    const instructions: Instruction[] = [new Jump(invalidJumpDestination)];

    const machineState = new AvmMachineState(initExecutionEnvironment({ calldata }));
    const interpreter = new AvmInterpreter(machineState, journal, instructions);

    const avmReturnData = await interpreter.run();

    expect(avmReturnData.reverted).toBe(true);
    expect(avmReturnData.revertReason).toBeInstanceOf(InvalidProgramCounterError);
    expect(avmReturnData.output).toHaveLength(0);
  });
});
