import { expect } from 'chai';
import { Package } from '../../src/noir/package';
import { FileManager } from '../../src/noir/file-manager/file-manager';

describe('Package', () => {
  describe('getSources', () => {
    it('should handle nested submodules correctly when package name matches module file', async () => {
      // Create a mock file manager
      const mockFiles = new Map([
        ['test-pkg/src/lib.nr', 'mod test_pkg;'],
        ['test-pkg/src/test_pkg.nr', 'pub mod constants;'],
        ['test-pkg/src/test_pkg/constants.nr', 'pub const FOO: Field = 42;'],
        ['test-pkg/src/other.nr', 'pub mod bar;'],
        ['test-pkg/src/other/bar.nr', 'pub fn baz() {}'],
      ]);

      const fm: FileManager = {
        readdir: async (path: string, _options?: { recursive?: boolean }) => {
          const results: string[] = [];
          for (const [file] of mockFiles) {
            if (file.startsWith(path + '/')) {
              results.push(file);
            }
          }
          return results;
        },
        readFile: async (path: string, _encoding?: string) => {
          return mockFiles.get(path) || '';
        },
      } as FileManager;

      const packageConfig = {
        package: { name: 'test_pkg', type: 'lib' as const },
        dependencies: {},
      };

      const pkg = new Package('test-pkg', 'test-pkg/src', packageConfig);
      const sources = await pkg.getSources(fm, 'test_pkg');

      // Convert to a map for easier testing
      const sourceMap = new Map(sources.map((s) => [s.path, s.source]));

      // Check that files are mapped correctly
      expect(sourceMap.has('test_pkg/lib.nr')).to.be.true;
      expect(sourceMap.has('test_pkg/test_pkg.nr')).to.be.true;
      // This is the key test - files in src/test_pkg/ should be mapped to test_pkg/ not test_pkg/test_pkg/
      expect(sourceMap.has('test_pkg/constants.nr')).to.be.true;
      expect(sourceMap.has('test_pkg/test_pkg/constants.nr')).to.be.false;
      // Other modules should keep their full path
      expect(sourceMap.has('test_pkg/other.nr')).to.be.true;
      expect(sourceMap.has('test_pkg/other/bar.nr')).to.be.true;
    });

    it('should not apply special handling when module name does not match package name', async () => {
      const mockFiles = new Map([
        ['lib-x/src/lib.nr', 'mod module_y;'],
        ['lib-x/src/module_y.nr', 'pub mod constants;'],
        ['lib-x/src/module_y/constants.nr', 'pub const FOO: Field = 42;'],
      ]);

      const fm: FileManager = {
        readdir: async (path: string, _options?: { recursive?: boolean }) => {
          const results: string[] = [];
          for (const [file] of mockFiles) {
            if (file.startsWith(path + '/')) {
              results.push(file);
            }
          }
          return results;
        },
        readFile: async (path: string, _encoding?: string) => {
          return mockFiles.get(path) || '';
        },
      } as FileManager;

      const packageConfig = {
        package: { name: 'lib_x', type: 'lib' as const },
        dependencies: {},
      };

      const pkg = new Package('lib-x', 'lib-x/src', packageConfig);
      const sources = await pkg.getSources(fm, 'lib_x');

      const sourceMap = new Map(sources.map((s) => [s.path, s.source]));

      // Since module_y doesn't match package name lib_x, no special handling should apply
      expect(sourceMap.has('lib_x/lib.nr')).to.be.true;
      expect(sourceMap.has('lib_x/module_y.nr')).to.be.true;
      expect(sourceMap.has('lib_x/module_y/constants.nr')).to.be.true;
      expect(sourceMap.has('lib_x/constants.nr')).to.be.false;
    });
  });
});
