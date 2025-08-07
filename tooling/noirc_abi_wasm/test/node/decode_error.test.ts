import { expect } from 'chai';
import { RawAssertionPayload, abiDecodeError } from '@noir-lang/noirc_abi';

it('Recovers custom field errors', async () => {
  const { FAKE_FIELD_SELECTOR, abi } = await import('../shared/decode_error');

  const payload: RawAssertionPayload = {
    selector: FAKE_FIELD_SELECTOR,
    data: ['0x000001'],
  };

  const decoded = abiDecodeError(abi, payload);
  expect(decoded).to.equal('0x01');
});

it('Recovers custom tuple errors', async () => {
  const { FAKE_TUPLE_SELECTOR, abi } = await import('../shared/decode_error');

  const payload: RawAssertionPayload = {
    selector: FAKE_TUPLE_SELECTOR,
    data: ['0x000001', '0x000002'],
  };

  const decoded = abiDecodeError(abi, payload);
  expect(decoded).to.deep.equal(['0x01', '0x02']);
});

it('Recovers custom fmt string errors', async () => {
  const { FAKE_FMT_STRING_SELECTOR, abi, SAMPLE_FMT_STRING } = await import('../shared/decode_error');

  // FmtStrings contain the string serialized to fields
  const data = [...SAMPLE_FMT_STRING].map((c) => `0x${c.charCodeAt(0).toString(16)}`);
  // Then they contain the length of the values to replace
  data.push('0x01');
  // And then the value to replace
  data.push('0x07');

  const payload: RawAssertionPayload = {
    selector: FAKE_FMT_STRING_SELECTOR,
    data,
  };

  const decoded = abiDecodeError(abi, payload);
  expect(decoded).to.equal('hello 0x07');
});

it('Recovers struct errors', async () => {
  const { FAKE_STRUCT_SELECTOR, abi } = await import('../shared/decode_error');

  const payload: RawAssertionPayload = {
    selector: FAKE_STRUCT_SELECTOR,
    data: ['0x01', '0x02'],
  };

  const decoded = abiDecodeError(abi, payload);
  expect(decoded).to.deep.equal({
    a: '0x01',
    b: '0x02',
  });
});
