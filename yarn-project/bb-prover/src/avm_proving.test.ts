import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { AvmSimulator } from '@aztec/simulator';
import { getAvmTestContractBytecode, initContext, initExecutionEnvironment } from '@aztec/simulator/avm/fixtures';

import fs from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'path';

import { type BBSuccess, BB_RESULT, generateAvmProof, verifyAvmProof } from './bb/execute.js';

const TIMEOUT = 30_000;

describe('AVM WitGen, proof generation and verification', () => {
  it(
    'Should prove valid execution contract function that performs addition',
    async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const internalLogger = createDebugLogger('aztec:avm-proving-test');
      const logger = (msg: string, _data?: any) => internalLogger.verbose(msg);
      const bytecode = getAvmTestContractBytecode('add_args_return');
      // The paths for the barretenberg binary and the write path are hardcoded for now.
      const bbPath = path.resolve('../../barretenberg/cpp/build/bin/bb');
      const bbWorkingDirectory = await fs.mkdtemp(path.join(tmpdir(), 'bb-'));

      // First we simulate (though it's not needed in this simple case).
      const results = await new AvmSimulator(context).executeBytecode(bytecode);
      expect(results.reverted).toBe(false);

      // Then we prove.
      const proofRes = await generateAvmProof(
        bbPath,
        bbWorkingDirectory,
        bytecode,
        context.environment.calldata,
        logger,
      );
      expect(proofRes.status).toEqual(BB_RESULT.SUCCESS);

      // Then we verify.
      const succeededRes = proofRes as BBSuccess;
      const verificationRes = await verifyAvmProof(bbPath, succeededRes.proofPath!, succeededRes.vkPath!, logger);
      expect(verificationRes.status).toBe(BB_RESULT.SUCCESS);
    },
    TIMEOUT,
  );
});
