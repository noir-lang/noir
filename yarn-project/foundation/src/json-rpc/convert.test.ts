import { Buffer } from 'buffer';

import { ClassConverter } from './class_converter.js';
import { convertFromJsonObj, convertToJsonObj } from './convert.js';
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
