import { Fr } from '@aztec/circuits.js';

import { parseFieldFromHexString } from './commands.js';

describe('parseFieldFromHexString', () => {
  it.each<[string, Fr]>([
    ['0', Fr.ZERO],
    ['0x0', Fr.ZERO],
    ['0x1', new Fr(1)],
    ['0x00', Fr.ZERO],
    ['fa', new Fr(0xfa)],
    ['123', new Fr(0x0123)],
    ['0xff', new Fr(255)],
    ['0x0000000000000000000000000000000000000000000000000000000000000003', new Fr(3)],
    ['0x00000000000000000000000000000000000000000000000000000000000000003', new Fr(3)],
  ])('parses the field %s correctly', (str, expected) => {
    expect(parseFieldFromHexString(str)).toEqual(expected);
  });
});
