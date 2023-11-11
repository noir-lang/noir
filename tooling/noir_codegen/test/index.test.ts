import { expect } from 'chai';
import { assert_lt, u64 } from './codegen/index.js';

it('codegens a callable function', async () => {
  const result: u64 = await assert_lt('2', '3', [0, 0, 0, 0, 0], { foo: true, bar: ['12345', '12345', '12345'] });

  expect(result).to.be.eq('0x05');
});
