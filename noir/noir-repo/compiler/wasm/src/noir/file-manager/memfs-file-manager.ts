import { IFs, fs } from 'memfs';
import { IDirent } from 'memfs/lib/node/types/misc';

import { FileManager } from './file-manager';

/**
 * Creates a new FileManager instance based on a MemFS instance
 * @param memFS - the memfs backing instance
 * @param dataDir - where to store files
 */
export function createMemFSFileManager(memFS: IFs = fs, dataDir = '/'): FileManager {
  const readdirRecursive = async (dir: string): Promise<string[]> => {
    const contents = await memFS.promises.readdir(dir);
    let files: string[] = [];
    for (const handle in contents) {
      if ((handle as unknown as IDirent).isFile()) {
        files.push(handle.toString());
      } else {
        files = files.concat(await readdirRecursive(handle.toString()));
      }
    }
    return files;
  };
  return new FileManager(
    {
      existsSync: memFS.existsSync.bind(memFS),
      mkdir: async (
        dir: string,
        options?: {
          /**
           * Traverse child directories
           */
          recursive: boolean;
        },
      ) => {
        await memFS.promises.mkdir(dir, options);
      },
      writeFile: memFS.promises.writeFile.bind(memFS),
      rename: memFS.promises.rename.bind(memFS),
      readFile: memFS.promises.readFile.bind(memFS),
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
        return (await memFS.promises.readdir(dir)).map((handles) => handles.toString());
      },
    },
    dataDir,
  );
}
