import { type DebugLogger } from '@aztec/aztec.js';
import { randomBytes } from '@aztec/foundation/crypto';

import * as fs from 'fs/promises';

export { deployAndInitializeTokenAndBridgeContracts } from '../shared/cross_chain_test_harness.js';

const {
  NOIR_RELEASE_DIR = 'noir-repo/target/release',
  TEMP_DIR = '/tmp',
  ACVM_BINARY_PATH = '',
  ACVM_WORKING_DIRECTORY = '',
  ACVM_FORCE_WASM = '',
} = process.env;

// Determines if we have access to the acvm binary and a tmp folder for temp files
export async function getACVMConfig(logger: DebugLogger): Promise<
  | {
      acvmWorkingDirectory: string;
      acvmBinaryPath: string;
      cleanup: () => Promise<void>;
    }
  | undefined
> {
  try {
    if (['1', 'true'].includes(ACVM_FORCE_WASM)) {
      return undefined;
    }
    const acvmBinaryPath = ACVM_BINARY_PATH ? ACVM_BINARY_PATH : `../../noir/${NOIR_RELEASE_DIR}/acvm`;
    await fs.access(acvmBinaryPath, fs.constants.R_OK);
    const tempWorkingDirectory = `${TEMP_DIR}/${randomBytes(4).toString('hex')}`;
    const acvmWorkingDirectory = ACVM_WORKING_DIRECTORY ? ACVM_WORKING_DIRECTORY : `${tempWorkingDirectory}/acvm`;
    await fs.mkdir(acvmWorkingDirectory, { recursive: true });
    logger.verbose(`Using native ACVM binary at ${acvmBinaryPath} with working directory ${acvmWorkingDirectory}`);

    const directoryToCleanup = ACVM_WORKING_DIRECTORY ? undefined : tempWorkingDirectory;

    const cleanup = async () => {
      if (directoryToCleanup) {
        // logger(`Cleaning up ACVM temp directory ${directoryToCleanup}`);
        await fs.rm(directoryToCleanup, { recursive: true, force: true });
      }
    };

    return {
      acvmWorkingDirectory,
      acvmBinaryPath,
      cleanup,
    };
  } catch (err) {
    logger.verbose(`Native ACVM not available, error: ${err}`);
    return undefined;
  }
}
