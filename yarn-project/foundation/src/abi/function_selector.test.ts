import { setupCustomSnapshotSerializers } from '../testing/index.js';
import { FunctionSelector } from './function_selector.js';

describe('FunctionSelector', () => {
  let selector: FunctionSelector;

  beforeAll(() => {
    setupCustomSnapshotSerializers(expect);
    selector = FunctionSelector.random();
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = selector.toBuffer();
    const res = FunctionSelector.fromBuffer(buffer);
    expect(res).toEqual(selector);
    expect(res.isEmpty()).toBe(false);
  });

  it('serializes to field and deserializes it back', () => {
    const field = selector.toField();
    const res = FunctionSelector.fromField(field);
    expect(res).toEqual(selector);
  });

  it('computes a function selector from signature', () => {
    const res = FunctionSelector.fromSignature('transfer(address,uint256)');
    expect(res).toMatchSnapshot();
  });
});
