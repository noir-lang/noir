import { Fr } from '@aztec/foundation/fields';
import { AvmTestContractArtifact } from '@aztec/noir-contracts';

import { mock } from 'jest-mock-extended';

import { AvmMachineState } from './avm_machine_state.js';
import { TypeTag } from './avm_memory_types.js';
import { initExecutionEnvironment } from './fixtures/index.js';
import { executeAvm } from './interpreter/interpreter.js';
import { AvmJournal } from './journal/journal.js';
import { Add, CalldataCopy, Return } from './opcodes/index.js';
import { decodeFromBytecode, encodeToBytecode } from './serialization/bytecode_serialization.js';

describe('avm', () => {
  it('Should execute bytecode that performs basic addition', async () => {
    const calldata: Fr[] = [new Fr(1), new Fr(2)];
    const journal = mock<AvmJournal>();

    // Construct bytecode
    const bytecode = encodeToBytecode([
      new CalldataCopy(/*indirect=*/ 0, /*cdOffset=*/ 0, /*copySize=*/ 2, /*dstOffset=*/ 0),
      new Add(/*indirect=*/ 0, TypeTag.FIELD, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Return(/*indirect=*/ 0, /*returnOffset=*/ 2, /*copySize=*/ 1),
    ]);

    // Decode bytecode into instructions
    const instructions = decodeFromBytecode(bytecode);

    // Execute instructions
    const context = new AvmMachineState(initExecutionEnvironment({ calldata }));
    const avmReturnData = await executeAvm(context, journal, instructions);

    expect(avmReturnData.reverted).toBe(false);

    const returnData = avmReturnData.output;
    expect(returnData.length).toBe(1);
    expect(returnData).toEqual([new Fr(3)]);
  });

  describe('testing transpiled Noir contracts', () => {
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/4361): sync wire format w/transpiler.
    it('Should execute contract function that performs addition', async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const journal = mock<AvmJournal>();

      // Get contract function artifact
      const addArtifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_addArgsReturn')!;

      // Decode bytecode into instructions
      const instructionsBytecode = Buffer.from(addArtifact.bytecode, 'base64');
      const instructions = decodeFromBytecode(instructionsBytecode);

      // Execute instructions
      const context = new AvmMachineState(initExecutionEnvironment({ calldata }));
      const avmReturnData = await executeAvm(context, journal, instructions);

      expect(avmReturnData.reverted).toBe(false);

      const returnData = avmReturnData.output;
      expect(returnData.length).toBe(1);
      expect(returnData).toEqual([new Fr(3)]);
    });
  });
});
