import { times } from './array.js';

describe('times', () => {
  it('should return an array with the result from all executions', () => {
    const result = times(5, i => i * 2);
    expect(result).toEqual([0, 2, 4, 6, 8]);
  });

  it('should return an empty array when n is 0', () => {
    const result = times(0, i => i * 2);
    expect(result).toEqual([]);
  });
});
