import { fileURLToPath } from '@aztec/foundation/url';

import { createFsFromVolume } from 'memfs';
import { Volume } from 'memfs/lib/volume.js';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';

import { FileManager } from '../file-manager/file-manager.js';
import { createMemFSFileManager } from '../file-manager/memfs-file-manager.js';
import { NoirPackage } from '../package.js';
import { NoirDependencyResolver } from './dependency-resolver.js';
import { LocalDependencyResolver } from './local-dependency-resolver.js';

describe('DependencyResolver', () => {
  let resolver: NoirDependencyResolver;
  let fm: FileManager;
  let pkg: NoirPackage;

  beforeEach(async () => {
    const fixtures = join(dirname(fileURLToPath(import.meta.url)), '../../../fixtures');
    const memFS = createFsFromVolume(new Volume());
    memFS.mkdirSync('/test_contract/src', { recursive: true });
    memFS.mkdirSync('/test_lib/src', { recursive: true });
    memFS.writeFileSync('/test_contract/Nargo.toml', await readFile(join(fixtures, 'test_contract/Nargo.toml')));
    memFS.writeFileSync('/test_contract/src/main.nr', await readFile(join(fixtures, 'test_contract/src/main.nr')));
    memFS.writeFileSync('/test_lib/Nargo.toml', await readFile(join(fixtures, 'test_lib/Nargo.toml')));
    memFS.writeFileSync('/test_lib/src/lib.nr', await readFile(join(fixtures, 'test_lib/src/lib.nr')));

    fm = createMemFSFileManager(memFS, '/');

    pkg = await NoirPackage.open('/test_contract', fm);
    resolver = new LocalDependencyResolver(fm);
  });

  it("returns null if it can't resolve a dependency", async () => {
    const dep = await resolver.resolveDependency(pkg, {
      git: 'git@some-git-host',
      directory: '/',
      tag: 'v1.0.0',
    });

    expect(dep).toBeNull();
  });

  it.each(['../test_contract', '/test_contract'])('resolves a known dependency', async path => {
    const lib = await resolver.resolveDependency(pkg, {
      path,
    });
    expect(lib).toBeDefined();
    expect(lib!.version).toBeUndefined();
    expect(fm.hasFileSync(lib!.package.getEntryPointPath())).toBe(true);
  });
});
