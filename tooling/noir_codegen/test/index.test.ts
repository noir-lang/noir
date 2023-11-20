import { expect } from 'chai';
import { assert_lt } from './codegen/index.js';

it('codegens a callable function', async () => {
  const result = await assert_lt({
    x: '2',
    y: '3',
  });

  expect(result).to.be.eq('0x05');
});
