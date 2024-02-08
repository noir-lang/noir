import { TaggedMemory, Uint32 } from '../avm_memory_types.js';
import { Addressing, AddressingMode } from './addressing_mode.js';

describe('Addressing', () => {
  it('should reserialize correctly', () => {
    const addressingMode = Addressing.fromWire(0b10101010);
    const wireModes = addressingMode.toWire();
    expect(wireModes).toBe(0b10101010);
  });

  it('should convert to wire format correctly', () => {
    const addressingMode = new Addressing([
      AddressingMode.INDIRECT,
      AddressingMode.DIRECT,
      AddressingMode.INDIRECT,
      AddressingMode.DIRECT,
    ]);
    const wireModes = addressingMode.toWire();
    expect(wireModes).toBe(0b00000101);
  });

  it('should convert from wire format correctly', () => {
    const addressingMode = Addressing.fromWire(0b00011001);
    const expected = new Addressing([
      AddressingMode.INDIRECT,
      AddressingMode.DIRECT,
      AddressingMode.DIRECT,
      AddressingMode.INDIRECT,
      AddressingMode.INDIRECT,
      AddressingMode.DIRECT,
      AddressingMode.DIRECT,
      AddressingMode.DIRECT,
    ]);

    expect(addressingMode).toStrictEqual(expected);
  });

  it('should resolve offsets correctly', () => {
    const addressingMode = Addressing.fromWire(0b00011001);
    const offsets = [10, 20, 30];
    const mem = new TaggedMemory();
    mem.set(10, new Uint32(100));
    mem.set(20, new Uint32(200));
    mem.set(30, new Uint32(300));

    const resolved = addressingMode.resolve(offsets, mem);
    expect(resolved).toEqual([100, 20, 30]);
  });
});
