import { createPXEClient, waitForPXE } from '@aztec/aztec.js';

import { execSync } from 'child_process';

const { PXE_URL = '' } = process.env;

// Entrypoint for running the up-quick-start script on the CI
describe('guides/up_quick_start', () => {
  // TODO: update to not use CLI
  it.skip('works', async () => {
    await waitForPXE(createPXEClient(PXE_URL));
    execSync(
      `DEBUG="aztec:*" PXE_URL=\${PXE_URL:-http://localhost:8080} PATH=$PATH:../node_modules/.bin ./src/guides/up_quick_start.sh`,
      {
        shell: '/bin/bash',
        stdio: 'inherit',
      },
    );
  });
});
