import { Add, Sub } from './arithmetic.js';
import { OPCODE_BYTE_LENGTH, OPERAND_BTYE_LENGTH, interpretBytecode } from './from_bytecode.js';
import { Instruction } from './instruction.js';

describe('Avm Interpreter', () => {
  const toByte = (num: number): Buffer => {
    const buf = Buffer.alloc(OPCODE_BYTE_LENGTH);
    buf.writeUInt8(num);
    return buf;
  };
  const to4Byte = (num: number): Buffer => {
    const buf = Buffer.alloc(OPERAND_BTYE_LENGTH);
    buf.writeUInt32BE(num);
    return buf;
  };

  it('Should read bytecode string into a list of opcodes', () => {
    const opcode = 1;
    const opcode2 = 2;
    const a = 1;
    const b = 2;
    const c = 3;

    const ops = toByte(opcode);
    const ops2 = toByte(opcode2);
    const as = to4Byte(a);
    const bs = to4Byte(b);
    const cs = to4Byte(c);
    const bytecode = Buffer.concat([ops, as, bs, cs, ops2, as, bs, cs]);

    const expectedInstructions: Instruction[] = [new Add(a, b, c), new Sub(a, b, c)];

    const instructions = interpretBytecode(bytecode);
    expect(instructions).toEqual(expectedInstructions);
  });
});
