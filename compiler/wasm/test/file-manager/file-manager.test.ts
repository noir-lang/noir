import { Volume, createFsFromVolume } from 'memfs';
import { existsSync, mkdtempSync, rmSync } from 'fs';
import * as fs from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';

import { FileManager, FileSystem } from '../../src/noir/file-manager/file-manager';
import { createMemFSFileManager } from '../../src/noir/file-manager/memfs-file-manager';

import { expect } from 'chai';
import forEach from 'mocha-each';

const memFS = (): { fm: FileManager; teardown: () => void } => {
  const fm = createMemFSFileManager(createFsFromVolume(new Volume()), '/');
  return {
    fm,
    // no-op, it's all in memory
    teardown: () => {},
  };
};

const nodeFS = (): { fm: FileManager; teardown: () => void } => {
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
forEach([
  ['memFs', memFS],
  ['nodeFS', nodeFS],
]).describe('FileManager: %s', (name, fs) => {
  let fm: FileManager;
  let testFileContent: string;
  let testFileBytes: Uint8Array;
  let teardown: () => void;

  beforeEach(() => {
    ({ fm, teardown } = fs());
    testFileContent = 'foo';
    testFileBytes = new TextEncoder().encode(testFileContent);
  });

  afterEach(() => {
    return teardown?.();
  });

  it(`saves files and correctly reads bytes back using ${name}`, async () => {
    await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
    expect(fm.readFile('test.txt')).to.eventually.eq(testFileBytes);
  });

  it(`saves files and correctly reads UTF-8 string back using ${name}`, async () => {
    await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
    expect(fm.readFile('test.txt', 'utf-8')).to.eventually.eq(testFileContent);
  });

  it(`correctly checks if file exists or not using ${name}`, async () => {
    expect(fm.hasFileSync('test.txt')).to.eq(false);
    await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
    expect(fm.hasFileSync('test.txt')).to.eq(true);
  });

  it(`moves files using ${name}`, async () => {
    await fm.writeFile('test.txt.tmp', new Blob([testFileBytes]).stream());
    expect(fm.hasFileSync('test.txt.tmp')).to.eq(true);

    await fm.moveFile('test.txt.tmp', 'test.txt');

    expect(fm.hasFileSync('test.txt.tmp')).to.eq(false);
    expect(fm.hasFileSync('test.txt')).to.eq(true);
  });
});
