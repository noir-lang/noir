import { Volume, createFsFromVolume } from 'memfs';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import * as fs from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { FileManager, FileSystem } from './file-manager.js';
import { createMemFSFileManager } from './memfs-file-manager.js';

const memFS = (): { fm: FileManager; teardown: () => void } => {
  const fm = createMemFSFileManager(createFsFromVolume(new Volume()), '/');
  return {
    fm,
    // no-op, it's all in memory
    teardown: () => {},
  };
};

const nodeFM = (): { fm: FileManager; teardown: () => void } => {
  const fileSystem: FileSystem = {
    existsSync: existsSync,
    mkdir: async (dir: string, opts?: { recursive: boolean }) => {
      await fs.mkdir(dir, opts);
    },
    writeFile: fs.writeFile,
    readFile: fs.readFile,
    rename: fs.rename,
    readdir: fs.readdir,
  };

  const dir = mkdtempSync(join(tmpdir(), 'noir-compiler-test'));
  const fm = new FileManager(fileSystem, dir);

  return {
    fm,
    teardown: () => {
      rmSync(dir, {
        recursive: true,
      });
    },
  };
};

/**
 * Declare the default test suite for a file manager
 * @param setup - Function to setup a file manager
 * @param teardown - Optional function to call at the end of the test
 */
describe.each([memFS, nodeFM])('FileManager', setup => {
  let fm: FileManager;
  let testFileContent: string;
  let testFileBytes: Uint8Array;
  let teardown: () => void;

  beforeEach(() => {
    ({ fm, teardown } = setup());
    testFileContent = 'foo';
    testFileBytes = new TextEncoder().encode(testFileContent);
  });

  afterEach(() => {
    return teardown?.();
  });

  it('saves files and correctly reads bytes back', async () => {
    await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
    await expect(fm.readFile('test.txt')).resolves.toEqual(testFileBytes);
  });

  it('saves files and correctly reads UTF-8 string back', async () => {
    await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
    await expect(fm.readFile('test.txt', 'utf-8')).resolves.toEqual(testFileContent);
  });

  it('correctly checks if file exists or not', async () => {
    expect(fm.hasFileSync('test.txt')).toBe(false);
    await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
    expect(fm.hasFileSync('test.txt')).toBe(true);
  });

  it('moves files', async () => {
    await fm.writeFile('test.txt.tmp', new Blob([testFileBytes]).stream());
    expect(fm.hasFileSync('test.txt.tmp')).toBe(true);

    await fm.moveFile('test.txt.tmp', 'test.txt');

    expect(fm.hasFileSync('test.txt.tmp')).toBe(false);
    expect(fm.hasFileSync('test.txt')).toBe(true);
  });
});
