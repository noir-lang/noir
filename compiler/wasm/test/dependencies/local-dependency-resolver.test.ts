import { createFsFromVolume, Volume } from 'memfs';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'path';

import { FileManager } from '../../src/noir/file-manager/file-manager';
import { createMemFSFileManager } from '../../src/noir/file-manager/memfs-file-manager';
import { NoirPackage } from '../../src/noir/package';
import { NoirDependencyResolver } from '../../src/noir/dependencies/dependency-resolver';
import { LocalDependencyResolver } from '../../src/noir/dependencies/local-dependency-resolver';
import { expect } from 'chai';
import forEach from 'mocha-each';

describe('DependencyResolver', () => {
  let resolver: NoirDependencyResolver;
  let fm: FileManager;
  let pkg: NoirPackage;

  beforeEach(async () => {
    const fixtures = join(__dirname, '../../public/fixtures');
    const memFS = createFsFromVolume(new Volume());
    memFS.mkdirSync('/noir-contract/src', { recursive: true });
    memFS.mkdirSync('/lib-c/src', { recursive: true });
    memFS.writeFileSync('/noir-contract/Nargo.toml', await readFile(join(fixtures, 'noir-contract/Nargo.toml')));
    memFS.writeFileSync('/noir-contract/src/main.nr', await readFile(join(fixtures, 'noir-contract/src/main.nr')));
    memFS.writeFileSync('/lib-c/Nargo.toml', await readFile(join(fixtures, 'deps/lib-c/Nargo.toml')));
    memFS.writeFileSync('/lib-c/src/lib.nr', await readFile(join(fixtures, 'deps/lib-c/src/lib.nr')));

    fm = createMemFSFileManager(memFS, '/');

    pkg = await NoirPackage.open('/noir-contract', fm);
    resolver = new LocalDependencyResolver(fm);
  });

  it("returns null if it can't resolve a dependency", async () => {
    const dep = await resolver.resolveDependency(pkg, {
      git: 'git@some-git-host',
      directory: '/',
      tag: 'v1.0.0',
    });

    expect(dep).to.be.null;
  });

  forEach(['../noir-contract', '/noir-contract']).it('resolves a known dependency %s', async (path) => {
    const lib = await resolver.resolveDependency(pkg, {
      path,
    });
    expect(lib).not.to.be.undefined;
    expect(lib!.version).to.be.undefined;
    expect(fm.hasFileSync(lib!.package.getEntryPointPath())).to.eq(true);
  });
});
