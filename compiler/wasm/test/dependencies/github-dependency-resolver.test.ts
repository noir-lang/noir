import { Volume, createFsFromVolume } from 'memfs';
import { dirname, join, resolve } from 'path';

import { FileManager } from '../../src/noir/file-manager/file-manager';
import { createMemFSFileManager } from '../../src/noir/file-manager/memfs-file-manager';
import { readdirRecursive } from '../../src/noir/file-manager/nodejs-file-manager';

import { Package } from '../../src/noir/package';
import { DependencyResolver } from '../../src/noir/dependencies/dependency-resolver';
import {
  GithubDependencyResolver,
  resolveGithubCodeArchive,
  safeFilename,
} from '../../src/noir/dependencies/github-dependency-resolver';
import { GitDependencyConfig } from '../../src/types/noir_package_config';
import Sinon, { SinonStub } from 'sinon';
import chai, { expect } from 'chai';
import forEach from 'mocha-each';
import chaiAsPromised from 'chai-as-promised';
import AdmZip from 'adm-zip';

chai.use(chaiAsPromised);

const fixtures = resolve(join(__dirname, '../fixtures'));

describe('GithubDependencyResolver', () => {
  let resolver: DependencyResolver;
  let fm: FileManager;
  let pkg: Package;
  let libDependency: GitDependencyConfig;
  let fetchStub: SinonStub | undefined;

  beforeEach(() => {
    fetchStub = Sinon.stub();
    fm = createMemFSFileManager(createFsFromVolume(new Volume()), '/');

    libDependency = {
      git: 'https://github.com/example/repo',
      tag: 'v1.0.0',
    };

    pkg = new Package('/test_contract', '/test_contract/src', {
      dependencies: {
        // eslint-disable-next-line camelcase
        lib_c: libDependency,
      },
      package: {
        name: 'test_contract',
        type: 'contract',
      },
    });

    resolver = new GithubDependencyResolver(fm, fetchStub);

    // cut off outside access
    fetchStub.onCall(0).throws(new Error());
  });

  it("returns null if it can't resolve a dependency", async () => {
    const dep = await resolver.resolveDependency(pkg, {
      path: '/lib-c',
    });

    expect(dep).to.be.null;
  });

  it('resolves Github dependency', async () => {
    const zip = new AdmZip();
    const testLibPath = join(fixtures, 'deps', 'lib-c');
    for (const filePath of await readdirRecursive(testLibPath)) {
      zip.addLocalFile(filePath, dirname(filePath.replace(testLibPath, 'lib-c')));
    }

    fetchStub?.onCall(0).returns(new Response(zip.toBuffer(), { status: 200 }));

    const lib = await resolver.resolveDependency(pkg, libDependency);
    expect(lib).not.to.be.undefined;
    expect(lib!.version).to.eq(libDependency.tag);
    expect(fm.hasFileSync(lib!.package.getEntryPointPath())).to.eq(true);
  });

  forEach([
    [
      'https://github.com/example/lib.nr/archive/v1.0.0.zip',
      'zip',
      {
        git: 'https://github.com/example/lib.nr',
        tag: 'v1.0.0',
      },
    ],
    [
      'https://github.com/example/lib.nr/archive/v1.0.0.tar.gz',
      'tar',
      {
        git: 'https://github.com/example/lib.nr',
        tag: 'v1.0.0',
      },
    ],
    [
      'https://github.com/example/lib.nr/archive/HEAD.zip',
      'zip',
      {
        git: 'https://github.com/example/lib.nr',
        tag: 'HEAD',
      },
    ],
    [
      'https://github.com/example/lib.nr/archive/HEAD.tar.gz',
      'tar',
      {
        git: 'https://github.com/example/lib.nr',
        tag: 'HEAD',
      },
    ],
  ]).it(
    'resolves to the correct code archive URL %s',
    async (href: string, format: 'zip' | 'tar', dep: GitDependencyConfig) => {
      const archiveUrl = resolveGithubCodeArchive(dep, format);
      expect(archiveUrl.href).to.eq(href);
    },
  );

  forEach([
    { git: 'https://github.com/', tag: 'v1' },
    { git: 'https://github.com/foo', tag: 'v1' },
    { git: 'https://example.com', tag: 'v1' },
  ]).it('throws if the Github URL is invalid %j', (dep) => {
    expect(() => resolveGithubCodeArchive(dep, 'zip')).to.throw();
  });

  forEach([
    ['main', 'main'],
    ['v1.0.0', 'v1.0.0'],
    ['../../../etc/passwd', '.._.._.._etc_passwd'],
    ['/etc/passwd', 'etc_passwd'],
    ['/SomeOrg/some-repo@v1.0.0', 'SomeOrg_some-repo@v1.0.0'],
    ['SomeOrg/some-repo@v1.0.0', 'SomeOrg_some-repo@v1.0.0'],
  ]).it('generates safe file names from %s', (value, expected) => {
    expect(safeFilename(value)).to.eq(expected);
  });

  forEach(['']).it('rejects invalid values', (value) => {
    expect(() => safeFilename(value)).to.throw();
  });
});
