import { encodeToBytecode } from './encode_to_bytecode.js';
import { AVM_OPCODE_BYTE_LENGTH, AVM_OPERAND_BYTE_LENGTH } from './instruction.js';
import { Opcode } from './opcodes.js';

describe('Avm Encoder', () => {
  const toByte = (num: number): Buffer => {
    const buf = Buffer.alloc(AVM_OPCODE_BYTE_LENGTH);
    buf.writeUInt8(num);
    return buf;
  };
  const to4Byte = (num: number): Buffer => {
    const buf = Buffer.alloc(AVM_OPERAND_BYTE_LENGTH);
    buf.writeUInt32BE(num);
    return buf;
  };

  it('Should properly encode instructions into bytecode buffers', () => {
    const addArgs = [0, 1, 2];
    const subArgs = [3, 4, 5];

    const addBytecode = encodeToBytecode(Opcode.ADD, addArgs);
    const subBytecode = encodeToBytecode(Opcode.SUB, subArgs);

    const expectedAddBytecode = Buffer.concat([toByte(Opcode.ADD), to4Byte(0), to4Byte(1), to4Byte(2)]);
    const expectedSubBytecode = Buffer.concat([toByte(Opcode.SUB), to4Byte(3), to4Byte(4), to4Byte(5)]);

    expect(addBytecode).toEqual(expectedAddBytecode);
    expect(subBytecode).toEqual(expectedSubBytecode);
  });
});
