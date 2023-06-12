import { NoirLogs } from './noir_logs.js';

describe('NoirLogs', () => {
  it('can encode NoirLogs to buffer and back', () => {
    const noirLogs = NoirLogs.random(42);

    const buffer = noirLogs.toBuffer();
    const recovered = NoirLogs.fromBuffer(buffer);

    expect(recovered).toEqual(noirLogs);
  });
});
