import * as fs from 'fs/promises';
import * as path from 'path';

// Create a random directory underneath a 'base' directory
// Calls a provided method, ensures the random directory is cleaned up afterwards
export async function runInDirectory<T>(
  workingDirBase: string,
  fn: (dir: string) => Promise<T>,
  cleanup: boolean = true,
): Promise<T> {
  // Create random directory to be used for temp files
  const workingDirectory = await fs.mkdtemp(path.join(workingDirBase, 'tmp-'));

  await fs.access(workingDirectory);

  try {
    return await fn(workingDirectory);
  } finally {
    if (cleanup) {
      await fs.rm(workingDirectory, { recursive: true, force: true });
    }
  }
}
