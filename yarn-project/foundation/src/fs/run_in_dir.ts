import { randomBytes } from 'crypto';
import * as fs from 'fs/promises';

// Create a random directory underneath a 'base' directory
// Calls a provided method, ensures the random directory is cleaned up afterwards
export async function runInDirectory<T>(workingDirBase: string, fn: (dir: string) => Promise<T>): Promise<T> {
  // Create random directory to be used for temp files
  const workingDirectory = `${workingDirBase}/${randomBytes(8).toString('hex')}`;
  await fs.mkdir(workingDirectory, { recursive: true });

  await fs.access(workingDirectory);

  try {
    return await fn(workingDirectory);
  } finally {
    await fs.rm(workingDirectory, { recursive: true, force: true });
  }
}
