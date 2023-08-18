import { Committable } from './committable.js';

describe('committable', () => {
  it('include uncommitted should work as expected', () => {
    const committableNumber: Committable<number> = new Committable(0);

    // Check the initial value is 0
    expect(committableNumber.get(true)).toBe(0);

    // Update the value to 1
    committableNumber.set(1);
    committableNumber.commit();

    // Check the value is 1
    expect(committableNumber.get()).toBe(1);

    // Test include uncommitted
    committableNumber.set(2);
    expect(committableNumber.get(true)).toBe(2);
    expect(committableNumber.get(false)).toBe(1);

    committableNumber.commit();
    expect(committableNumber.get()).toBe(2);

    // Test rollback
    committableNumber.set(3);
    expect(committableNumber.get(true)).toBe(3);
    expect(committableNumber.get(false)).toBe(2);
    committableNumber.rollback();
    expect(committableNumber.get()).toBe(2);
  });
});
