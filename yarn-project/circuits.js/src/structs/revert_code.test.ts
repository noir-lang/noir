import { RevertCode } from './revert_code.js';

describe('revert_code', () => {
  it.each([RevertCode.OK, RevertCode.REVERTED])('should serialize properly', revertCode => {
    const buf = revertCode.toBuffer();
    expect(buf).toMatchSnapshot();
    expect(RevertCode.fromBuffer(buf)).toEqual(revertCode);
  });
});
