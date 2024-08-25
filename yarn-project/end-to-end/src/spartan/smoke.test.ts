import { type PXE, createCompatibleClient } from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation/log';

const { PXE_URL } = process.env;
if (!PXE_URL) {
  throw new Error('PXE_URL env variable must be set');
}
const debugLogger = createDebugLogger('aztec:spartan-test');
// const userLog = createConsoleLogger();

describe('sample test', () => {
  let pxe: PXE;
  beforeAll(async () => {
    pxe = await createCompatibleClient(PXE_URL, debugLogger);
  });
  it('should be able to get node enr', async () => {
    const info = await pxe.getNodeInfo();
    expect(info).toBeDefined();
    // expect enr to be a string starting with 'enr:-'
    expect(info.enr).toMatch(/^enr:-/);
  });
});
