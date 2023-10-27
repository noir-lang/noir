import { IFs, fs } from 'memfs';

import { FileManager } from './file-manager.js';

/**
 * Creates a new FileManager instance based on a MemFS instance
 * @param memFS - the memfs backing instance
 * @param dataDir - where to store files
 */
export function createMemFSFileManager(memFS: IFs = fs, dataDir = '/'): FileManager {
  return new FileManager(
    {
      existsSync: memFS.existsSync.bind(memFS),
      mkdirSync: memFS.mkdirSync.bind(memFS),
      writeFileSync: memFS.writeFileSync.bind(memFS),
      renameSync: memFS.renameSync.bind(memFS),
      readFileSync: memFS.readFileSync.bind(memFS),
    },
    dataDir,
  );
}
