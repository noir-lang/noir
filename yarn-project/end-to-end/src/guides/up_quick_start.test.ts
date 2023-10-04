import { waitForSandbox } from '@aztec/aztec.js';

import { execSync } from 'child_process';

// Entrypoint for running the up-quick-start script on the CI
describe('guides/up_quick_start', () => {
  it('works', async () => {
    await waitForSandbox();
    execSync(
      `DEBUG="aztec:*" PXE_URL=\${PXE_URL:-http://localhost:8080} PATH=$PATH:../node_modules/.bin ./src/guides/up_quick_start.sh`,
      {
        shell: '/bin/bash',
        stdio: 'inherit',
      },
    );
  }, 90_000);
});
