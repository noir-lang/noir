import { type DebugLogger, fileURLToPath } from '@aztec/aztec.js';

import fs from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'path';

const {
  BB_RELEASE_DIR = 'barretenberg/cpp/build/bin',
  BB_BINARY_PATH,
  TEMP_DIR = tmpdir(),
  BB_WORKING_DIRECTORY = '',
} = process.env;

export const getBBConfig = async (
  logger: DebugLogger,
): Promise<{ bbBinaryPath: string; bbWorkingDirectory: string; cleanup: () => Promise<void> } | undefined> => {
  try {
    const bbBinaryPath =
      BB_BINARY_PATH ??
      path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../', BB_RELEASE_DIR, 'bb');
    await fs.access(bbBinaryPath, fs.constants.R_OK);

    let bbWorkingDirectory: string;
    let directoryToCleanup: string | undefined;

    if (BB_WORKING_DIRECTORY) {
      bbWorkingDirectory = BB_WORKING_DIRECTORY;
    } else {
      bbWorkingDirectory = await fs.mkdtemp(path.join(TEMP_DIR, 'bb-'));
      directoryToCleanup = bbWorkingDirectory;
    }

    await fs.mkdir(bbWorkingDirectory, { recursive: true });

    const cleanup = async () => {
      if (directoryToCleanup) {
        await fs.rm(directoryToCleanup, { recursive: true, force: true });
      }
    };

    return { bbBinaryPath, bbWorkingDirectory, cleanup };
  } catch (err) {
    logger.error(`Native BB not available, error: ${err}`);
    return undefined;
  }
};
