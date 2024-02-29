import { existsSync } from 'fs';
import { promises as fs } from 'fs';

import { FileManager } from './file-manager';

// This is needed because memfs doesn't support the recursive flag yet
export async function readdirRecursive(dir: string): Promise<string[]> {
  const contents = await fs.readdir(dir);
  let files: string[] = [];
  for (const handle of contents) {
    if ((await fs.stat(`${dir}/${handle}`)).isFile()) {
      files.push(`${dir}/${handle.toString()}`);
    } else {
      files = files.concat(await readdirRecursive(`${dir}/${handle.toString()}`));
    }
  }
  return files;
}

/**
 * Creates a new FileManager instance based on fs in node and memfs in the browser (via webpack alias)
 *
 * @param dataDir - root of the file system
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
        readdir: async (
          dir: string,
          options?: {
            /**
             * Traverse child directories
             */
            recursive: boolean;
          },
        ) => {
          if (options?.recursive) {
            return readdirRecursive(dir);
          }
          return (await fs.readdir(dir)).map((handles) => handles.toString());
        },
      },
    },
    dataDir,
  );
}
