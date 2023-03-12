import { fromBaseUnits, toBaseUnits } from './units.js';

describe('units', () => {
  it('should format correctly', () => {
    expect(fromBaseUnits(0n, 4, 2)).toBe('0');
    expect(fromBaseUnits(1299n, 4, 2)).toBe('0.12');
    expect(fromBaseUnits(198765n, 4, 2)).toBe('19.87');
    expect(fromBaseUnits(191111n, 4, 2)).toBe('19.11');
    expect(fromBaseUnits(100000n, 4, 2)).toBe('10');
    expect(fromBaseUnits(199999n, 4, 2)).toBe('19.99');
    expect(fromBaseUnits(199000n, 4, 2)).toBe('19.9');
    expect(fromBaseUnits(198765n, 4)).toBe('19.8765');
    expect(fromBaseUnits(190765n, 4, 6)).toBe('19.0765');
    expect(fromBaseUnits(-100n, 4, 2)).toBe('-0.01');
    expect(fromBaseUnits(198700n, 4, 6)).toBe('19.87');
    expect(fromBaseUnits(3000000000n, 6, 6)).toBe('3000');
  });

  it('should convert string to bigint correctly', () => {
    expect(toBaseUnits('0.0', 4)).toBe(0n);
    expect(toBaseUnits('0', 4)).toBe(0n);
    expect(toBaseUnits('', 4)).toBe(0n);
    expect(toBaseUnits('.', 4)).toBe(0n);
    expect(toBaseUnits('0.001', 3)).toBe(1n);
    expect(toBaseUnits('0.1299', 4)).toBe(1299n);
    expect(toBaseUnits('.1299', 4)).toBe(1299n);
    expect(toBaseUnits('0.1299', 3)).toBe(129n);
    expect(toBaseUnits('0.1299', 1)).toBe(1n);
    expect(toBaseUnits('12.34', 3)).toBe(12340n);
    expect(toBaseUnits('12.0', 3)).toBe(12000n);
    expect(toBaseUnits('12', 3)).toBe(12000n);
    expect(toBaseUnits('12.34', 0)).toBe(12n);
  });
});
