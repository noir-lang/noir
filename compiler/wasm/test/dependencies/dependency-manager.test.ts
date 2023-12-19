import { NoirDependencyConfig } from 'src/types/noir_package_config';
import { NoirPackage } from '../../src/noir/package';
import { NoirDependencyManager } from '../../src/noir/dependencies/dependency-manager';
import { NoirDependency, NoirDependencyResolver } from '../../src/noir/dependencies/dependency-resolver';

import { expect } from 'chai';

describe('DependencyManager', () => {
  let manager: NoirDependencyManager;

  beforeEach(() => {
    manager = new NoirDependencyManager(
      [new TestDependencyResolver()],
      new NoirPackage('/test_contract', '/test_contract/src', {
        dependencies: {
          lib1: {
            path: '/lib1',
          },
          lib2: {
            path: '/lib2',
          },
          lib3: {
            path: '/lib3',
          },
        },
        package: {
          name: 'test_contract',
          type: 'contract',
        },
      }),
    );
  });

  it('successfully resolves dependencies', async () => {
    await expect(manager.resolveDependencies()).to.eventually.be.undefined;
  });

  it('resolves all libraries', async () => {
    await manager.resolveDependencies();
    expect(manager.getPackageNames()).to.eql(['lib1', 'lib2', 'lib3']);
  });

  it('resolves root dependencies', async () => {
    await manager.resolveDependencies();
    expect(manager.getEntrypointDependencies()).to.eql(['lib1', 'lib2', 'lib3']);
  });

  it('resolves library dependencies', async () => {
    await manager.resolveDependencies();
    expect(manager.getLibraryDependencies()).to.eql({
      lib2: ['lib3'],
    });
  });
});

class TestDependencyResolver implements NoirDependencyResolver {
  // eslint-disable-next-line require-await
  public async resolveDependency(pkg: NoirPackage, dep: NoirDependencyConfig): Promise<NoirDependency | null> {
    if (!('path' in dep)) {
      return null;
    }

    switch (dep.path) {
      case '/lib1':
        return {
          version: '',
          package: new NoirPackage('/lib1', '/lib1/src', {
            dependencies: {},
            package: {
              name: 'lib1',
              type: 'lib',
            },
          }),
        };

      case '/lib2':
        return {
          version: '',
          package: new NoirPackage('/lib2', '/lib2/src', {
            dependencies: {
              lib3: {
                path: '../lib3',
              },
            },
            package: {
              name: 'lib2',
              type: 'lib',
            },
          }),
        };

      case '/lib3':
        return {
          version: '',
          package: new NoirPackage('/lib3', '/lib3/src', {
            dependencies: {},
            package: {
              name: 'lib3',
              type: 'lib',
            },
          }),
        };

      default:
        throw new Error();
    }
  }
}
