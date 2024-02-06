import { Field, Uint8 } from './avm_memory_types.js';

// TODO: complete
describe('Uint8', () => {
  it('Unsigned 8 max value', () => {
    expect(new Uint8(255).toBigInt()).toEqual(255n);
  });

  it('Unsigned 8 bit add', () => {
    expect(new Uint8(50).add(new Uint8(20))).toEqual(new Uint8(70));
  });

  it('Unsigned 8 bit add wraps', () => {
    expect(new Uint8(200).add(new Uint8(100))).toEqual(new Uint8(44));
  });
});

describe('Field', () => {
  it('Add correctly without wrapping', () => {
    expect(new Field(27).add(new Field(48))).toEqual(new Field(75));
  });
});
