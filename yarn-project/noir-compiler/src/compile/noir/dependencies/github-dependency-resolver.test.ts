import { fileURLToPath } from '@aztec/foundation/url';

import { jest } from '@jest/globals';
import { Volume, createFsFromVolume } from 'memfs';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';

import { FileManager } from '../file-manager/file-manager.js';
import { createMemFSFileManager } from '../file-manager/memfs-file-manager.js';
import { NoirGitDependencyConfig } from '../package-config.js';
import { NoirPackage } from '../package.js';
import { DependencyResolver } from './dependency-resolver.js';
import { GithubDependencyResolver, resolveGithubCodeArchive, safeFilename } from './github-dependency-resolver.js';

const fixtures = join(dirname(fileURLToPath(import.meta.url)), '../../../fixtures');

describe('GithubDependencyResolver', () => {
  let resolver: DependencyResolver;
  let fm: FileManager;
  let pkg: NoirPackage;
  let libDependency: NoirGitDependencyConfig;
  let fetchMock: jest.SpiedFunction<typeof fetch>;

  beforeEach(() => {
    fm = createMemFSFileManager(createFsFromVolume(new Volume()), '/');

    libDependency = {
      git: 'https://github.com/example/repo',
      tag: 'v1.0.0',
    };

    pkg = new NoirPackage('/test_contract', '/test_contract/src', {
      dependencies: {
        // eslint-disable-next-line camelcase
        test_lib: libDependency,
      },
      package: {
        name: 'test_contract',
        type: 'contract',
      },
    });

    resolver = new GithubDependencyResolver(fm);

    // cut off outside access
    fetchMock = jest.spyOn(globalThis, 'fetch').mockImplementation(() => {
      throw new Error();
    });
  });

  afterEach(() => {
    fetchMock.mockRestore();
  });

  it("returns null if it can't resolve a dependency", async () => {
    const dep = await resolver.resolveDependency(pkg, {
      path: '/test_lib',
    });

    expect(dep).toBeNull();
  });

  it('resolves Github dependency', async () => {
    fetchMock.mockResolvedValueOnce(new Response(await readFile(join(fixtures, 'test_lib.zip')), { status: 200 }));
    const libPkg = await resolver.resolveDependency(pkg, libDependency);
    expect(libPkg).toBeDefined();
    expect(fm.hasFileSync(libPkg!.getEntryPointPath())).toBe(true);
  });

  it.each<[NoirGitDependencyConfig, 'zip' | 'tar', string]>([
    [
      {
        git: 'https://github.com/example/lib.nr',
        tag: 'v1.0.0',
      },
      'zip',
      'https://github.com/example/lib.nr/archive/v1.0.0.zip',
    ],
    [
      {
        git: 'https://github.com/example/lib.nr',
        tag: 'v1.0.0',
      },
      'tar',
      'https://github.com/example/lib.nr/archive/v1.0.0.tar.gz',
    ],
    [
      {
        git: 'https://github.com/example/lib.nr',
      },
      'zip',
      'https://github.com/example/lib.nr/archive/HEAD.zip',
    ],
    [
      {
        git: 'https://github.com/example/lib.nr',
      },
      'tar',
      'https://github.com/example/lib.nr/archive/HEAD.tar.gz',
    ],
  ])('resolves to the correct code archive URL', (dep, format, href) => {
    const archiveUrl = resolveGithubCodeArchive(dep, format);
    expect(archiveUrl.href).toEqual(href);
  });

  it.each([{ git: 'https://github.com/' }, { git: 'https://github.com/foo' }, { git: 'https://example.com' }])(
    'throws if the Github URL is invalid',
    dep => {
      expect(() => resolveGithubCodeArchive(dep, 'zip')).toThrow();
    },
  );

  it.each([
    ['main', 'main'],
    ['v1.0.0', 'v1.0.0'],
    ['../../../etc/passwd', '.._.._.._etc_passwd'],
    ['/etc/passwd', 'etc_passwd'],
    ['/SomeOrg/some-repo@v1.0.0', 'SomeOrg_some-repo@v1.0.0'],
    ['SomeOrg/some-repo@v1.0.0', 'SomeOrg_some-repo@v1.0.0'],
  ])('generates safe file names', (value, expected) => {
    expect(safeFilename(value)).toEqual(expected);
  });

  it.each([''])('rejects invalid values', value => {
    expect(() => safeFilename(value)).toThrow();
  });
});
