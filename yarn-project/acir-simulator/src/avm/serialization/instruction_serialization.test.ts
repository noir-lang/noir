import { BufferCursor } from './buffer_cursor.js';
import { OperandType, deserialize, serialize } from './instruction_serialization.js';

class InstA {
  constructor(private a: number, private b: number, private c: number, private d: bigint, private e: bigint) {}

  static readonly opcode: number = 1;
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT16,
    OperandType.UINT32,
    OperandType.UINT64,
    OperandType.UINT128,
  ];
}

describe('Instruction Serialization', () => {
  it('Should serialize all types from OperandType[]', () => {
    const instance = new InstA(0x12, 0x1234, 0x12345678, 0x1234567887654321n, 0x1234567887654321abcdef0000fedcban);
    const actual: Buffer = serialize(InstA.wireFormat, instance);

    expect(actual).toEqual(
      Buffer.from(
        [
          // opcode
          '01',
          // a
          '12',
          // b
          '1234',
          // c
          '12345678',
          // d
          '1234567887654321',
          // e
          '1234567887654321ABCDEF0000FEDCBA',
        ].join(''),
        'hex',
      ),
    );
  });

  it('Should deserialize all types from OperandType[]', () => {
    const buffer = Buffer.from(
      [
        // opcode
        '01',
        // a
        '12',
        // b
        '1234',
        // c
        '12345678',
        // d
        '1234567887654321',
        // e
        '1234567887654321ABCDEF0000FEDCBA',
      ].join(''),
      'hex',
    );

    const deserializedParams = deserialize(new BufferCursor(buffer), InstA.wireFormat);
    const params = deserializedParams.slice(1) as ConstructorParameters<typeof InstA>; // Drop opcode.

    const actual = new InstA(...params);
    const expected = new InstA(0x12, 0x1234, 0x12345678, 0x1234567887654321n, 0x1234567887654321abcdef0000fedcban);
    expect(actual).toEqual(expected);
  });
});
