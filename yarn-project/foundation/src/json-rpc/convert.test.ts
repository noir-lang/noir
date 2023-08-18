import { Buffer } from 'buffer';

import { ClassConverter } from './class_converter.js';
import { convertBigintsInObj, convertFromJsonObj, convertToJsonObj } from './convert.js';
import { TestNote } from './fixtures/test_state.js';

const TEST_BASE64 = 'YmFzZTY0IGRlY29kZXI=';
test('test an RPC function over client', () => {
  const cc = new ClassConverter({ TestNote });
  const buffer = Buffer.from(TEST_BASE64, 'base64');
  expect(convertFromJsonObj(cc, convertToJsonObj(cc, buffer)).toString('base64')).toBe(TEST_BASE64);
  const note = new TestNote('1');
  expect(convertFromJsonObj(cc, convertToJsonObj(cc, note))).toBeInstanceOf(TestNote);
  expect(convertFromJsonObj(cc, convertToJsonObj(cc, note)).toString()).toBe('1');
});

test('converts a bigint', () => {
  expect(convertBigintsInObj(10n)).toEqual({ type: 'bigint', data: '10' });
  expect(convertBigintsInObj({ value: 10n })).toEqual({ value: { type: 'bigint', data: '10' } });
  expect(convertBigintsInObj([10n])).toEqual([{ type: 'bigint', data: '10' }]);
});

test('does not convert a string', () => {
  expect(convertBigintsInObj('hello')).toEqual('hello');
  expect(convertBigintsInObj({ msg: 'hello' })).toEqual({ msg: 'hello' });
});
