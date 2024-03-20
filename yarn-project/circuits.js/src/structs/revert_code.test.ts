import { Fr } from '@aztec/foundation/fields';

import { RevertCode } from './revert_code.js';

describe('revert_code', () => {
  it.each([RevertCode.OK, RevertCode.REVERTED])('should serialize properly', revertCode => {
    expect(revertCode.getSerializedLength()).toBe(1);

    const hashPreimage = revertCode.toHashPreimage();
    expect(hashPreimage).toMatchSnapshot();
    expect(hashPreimage.length).toBe(32);

    const buf = revertCode.toBuffer();
    expect(buf).toMatchSnapshot();
    expect(RevertCode.fromBuffer(buf)).toEqual(revertCode);

    const field = revertCode.toField();
    expect(field).toMatchSnapshot();
    expect(RevertCode.fromField(field)).toEqual(revertCode);
    expect(RevertCode.fromFields([field])).toEqual(revertCode);
  });

  it('should throw when deserializing from invalid buffer', () => {
    expect(() => RevertCode.fromBuffer(Buffer.from([42]))).toThrow();
    expect(() => RevertCode.fromField(new Fr(42))).toThrow();
  });
});
