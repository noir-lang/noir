import { existsSync } from 'node:fs';
import * as fs from 'node:fs/promises';

import { FileManager } from './file-manager.js';

/**
 * Creates a new FileManager instance based on nodejs fs
 * @param dataDir - where to store files
 */
export function createNodejsFileManager(dataDir: string): FileManager {
  return new FileManager(
    {
      ...fs,
      ...{
        // ExistsSync is not available in the fs/promises module
        existsSync,
        // This is added here because the node types are not compatible with the FileSystem type for mkdir
        // Typescripts tries to use a different variant of the function that is not the one that has the optional options.
        mkdir: async (
          dir: string,
          opts?: {
            /**
             * Traverse child directories
             */
            recursive: boolean;
          },
        ) => {
          await fs.mkdir(dir, opts);
        },
      },
    },
    dataDir,
  );
}
