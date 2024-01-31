import { Fr } from '@aztec/foundation/fields';
import { AvmTestContractArtifact } from '@aztec/noir-contracts';

import { mock } from 'jest-mock-extended';

import { AvmMachineState } from './avm_machine_state.js';
import { initExecutionEnvironment } from './fixtures/index.js';
import { executeAvm } from './interpreter/interpreter.js';
import { AvmJournal } from './journal/journal.js';
import { decodeBytecode } from './opcodes/decode_bytecode.js';
import { encodeToBytecode } from './opcodes/encode_to_bytecode.js';
import { Opcode } from './opcodes/opcodes.js';

describe('avm', () => {
  it('Should execute bytecode that performs basic addition', async () => {
    const calldata: Fr[] = [new Fr(1), new Fr(2)];
    const journal = mock<AvmJournal>();

    // Construct bytecode
    const calldataCopyArgs = [0, 2, 0];
    const addArgs = [0, 1, 2];
    const returnArgs = [2, 1];

    const calldataCopyBytecode = encodeToBytecode(Opcode.CALLDATACOPY, calldataCopyArgs);
    const addBytecode = encodeToBytecode(Opcode.ADD, addArgs);
    const returnBytecode = encodeToBytecode(Opcode.RETURN, returnArgs);
    const fullBytecode = Buffer.concat([calldataCopyBytecode, addBytecode, returnBytecode]);

    // Decode bytecode into instructions
    const instructions = decodeBytecode(fullBytecode);

    // Execute instructions
    const context = new AvmMachineState(initExecutionEnvironment({ calldata }));
    const avmReturnData = await executeAvm(context, journal, instructions);

    expect(avmReturnData.reverted).toBe(false);

    const returnData = avmReturnData.output;
    expect(returnData.length).toBe(1);
    expect(returnData).toEqual([new Fr(3)]);
  });

  describe('testing transpiled Noir contracts', () => {
    it('Should execute contract function that performs addition', async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const journal = mock<AvmJournal>();

      // Get contract function artifact
      const addArtifact = AvmTestContractArtifact.functions.find(f => f.name === 'avm_addArgsReturn')!;

      // Decode bytecode into instructions
      const instructions = decodeBytecode(Buffer.from(addArtifact.bytecode, 'base64'));

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
