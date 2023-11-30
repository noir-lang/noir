import { expect } from 'chai';
import { assert_lt, MyStruct, u64 } from './codegen/index.js';

it('codegens a callable function', async () => {
  const [sum, constant, struct]: [u64, u64, MyStruct] = await assert_lt(
    '2',
    '3',
    [0, 0, 0, 0, 0],
    { foo: true, bar: ['12345', '12345', '12345'] },
    '12345',
  );

  expect(sum).to.be.eq('0x05');
  expect(constant).to.be.eq('0x03');
  expect(struct).to.be.deep.eq({ foo: true, bar: ['12345', '12345', '12345'] });
});
