// import { Volume, createFsFromVolume } from 'memfs';
// import * as fs from 'node:fs';
// import { tmpdir } from 'node:os';
// import { join } from 'node:path';

// import { FileManager, FileSystem } from './file-manager.js';
// import { createMemFSFileManager } from './memfs-file-manager.js';

// const memFS = (): { fm: FileManager; teardown: () => void } => {
//   const fm = createMemFSFileManager(createFsFromVolume(new Volume()), '/');
//   return {
//     fm,
//     // no-op, it's all in memory
//     teardown: () => {},
//   };
// };

// const nodeFM = (): { fm: FileManager; teardown: () => void } => {
//   const fileSystem: FileSystem = {
//     existsSync: fs.existsSync,
//     mkdirSync: fs.mkdirSync,
//     writeFileSync: fs.writeFileSync,
//     readFileSync: fs.readFileSync,
//     renameSync: fs.renameSync,
//   };

//   const dir = fs.mkdtempSync(join(tmpdir(), 'noir-compiler-test'));
//   const fm = new FileManager(fileSystem, dir);

//   return {
//     fm,
//     teardown: () => {
//       fs.rmSync(dir, {
//         recursive: true,
//       });
//     },
//   };
// };

// /**
//  * Declare the default test suite for a file manager
//  * @param setup - Function to setup a file manager
//  * @param teardown - Optional function to call at the end of the test
//  */
// describe.each([memFS, nodeFM])('FileManager', setup => {
//   let fm: FileManager;
//   let testFileContent: string;
//   let testFileBytes: Uint8Array;
//   let teardown: () => void;

//   beforeEach(() => {
//     ({ fm, teardown } = setup());
//     testFileContent = 'foo';
//     testFileBytes = new TextEncoder().encode(testFileContent);
//   });

//   afterEach(() => {
//     return teardown?.();
//   });

//   it('saves files and correctly reads bytes back', async () => {
//     await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
//     expect(fm.readFileSync('test.txt')).toEqual(testFileBytes);
//   });

//   it('saves files and correctly reads UTF-8 string back', async () => {
//     await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
//     expect(fm.readFileSync('test.txt', 'utf-8')).toEqual(testFileContent);
//   });

//   it('correctly checks if file exists or not', async () => {
//     expect(fm.hasFileSync('test.txt')).toBe(false);
//     await fm.writeFile('test.txt', new Blob([testFileBytes]).stream());
//     expect(fm.hasFileSync('test.txt')).toBe(true);
//   });

//   it('moves files', async () => {
//     await fm.writeFile('test.txt.tmp', new Blob([testFileBytes]).stream());
//     expect(fm.hasFileSync('test.txt.tmp')).toBe(true);

//     fm.moveFileSync('test.txt.tmp', 'test.txt');

//     expect(fm.hasFileSync('test.txt.tmp')).toBe(false);
//     expect(fm.hasFileSync('test.txt')).toBe(true);
//   });
// });
