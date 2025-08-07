import { expect } from '@esm-bundle/chai';
import initNoirAbi, { RawAssertionPayload, abiDecodeError } from '@noir-lang/noirc_abi';

beforeEach(async () => {
  await initNoirAbi();
});

it('Recovers custom field errors', async () => {
  const { FAKE_FIELD_SELECTOR, abi } = await import('../shared/decode_error');

  const payload: RawAssertionPayload = {
    selector: FAKE_FIELD_SELECTOR,
    data: [Uint8Array.from(Buffer.from('0x0000000000000000000000000000000000000000000000000000000000000001', 'hex'))],
  };

  const decoded = abiDecodeError(abi, payload);
  expect(decoded).to.equal('0x01');
});

it('Recovers custom tuple errors', async () => {
  const { FAKE_TUPLE_SELECTOR, abi } = await import('../shared/decode_error');

  const payload: RawAssertionPayload = {
    selector: FAKE_TUPLE_SELECTOR,
    data: [
      Uint8Array.from(Buffer.from('0x0000000000000000000000000000000000000000000000000000000000000001', 'hex')),
      Uint8Array.from(Buffer.from('0x0000000000000000000000000000000000000000000000000000000000000002', 'hex')),
    ],
  };

  const decoded = abiDecodeError(abi, payload);
  expect(decoded).to.deep.equal(['0x01', '0x02']);
});

it('Recovers custom fmt string errors', async () => {
  const { FAKE_FMT_STRING_SELECTOR, abi, SAMPLE_FMT_STRING } = await import('../shared/decode_error');

  // FmtStrings contain the string serialized to fields
  const data = [...SAMPLE_FMT_STRING].map((c) =>
    Uint8Array.from(Buffer.from(`0x${c.charCodeAt(0).toString(16)}`, 'hex')),
  );
  // Then they contain the length of the values to replace
  data.push(Uint8Array.from(Buffer.from('0x0000000000000000000000000000000000000000000000000000000000000001', 'hex')));
  // And then the value to replace
  data.push(Uint8Array.from(Buffer.from('0x0000000000000000000000000000000000000000000000000000000000000007', 'hex')));

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
    data: [
      Uint8Array.from(Buffer.from('0x0000000000000000000000000000000000000000000000000000000000000001', 'hex')),
      Uint8Array.from(Buffer.from('0x0000000000000000000000000000000000000000000000000000000000000002', 'hex')),
    ],
  };

  const decoded = abiDecodeError(abi, payload);
  expect(decoded).to.deep.equal({
    a: '0x01',
    b: '0x02',
  });
});
