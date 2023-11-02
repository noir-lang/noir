import { NoirDependencyConfig } from '@aztec/foundation/noir';

import { NoirPackage } from '../package.js';
import { NoirDependencyManager } from './dependency-manager.js';
import { DependencyResolver } from './dependency-resolver.js';

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
        },
        package: {
          name: 'test_contract',
          type: 'contract',
        },
      }),
    );
  });

  it('successfully resolves dependencies', async () => {
    await expect(manager.resolveDependencies()).resolves.toBeUndefined();
  });

  it('resolves all libraries', async () => {
    await manager.resolveDependencies();
    expect(manager.getPackageNames()).toEqual(['lib1', 'lib2', 'lib3']);
  });

  it('resolves root dependencies', async () => {
    await manager.resolveDependencies();
    expect(manager.getEntrypointDependencies()).toEqual(['lib1', 'lib2']);
  });

  it('resolves library dependencies', async () => {
    await manager.resolveDependencies();
    expect(manager.getLibraryDependencies()).toEqual({
      lib2: ['lib3'],
    });
  });
});

class TestDependencyResolver implements DependencyResolver {
  // eslint-disable-next-line require-await
  public async resolveDependency(pkg: NoirPackage, dep: NoirDependencyConfig): Promise<NoirPackage | null> {
    if (!('path' in dep)) {
      return null;
    }

    switch (dep.path) {
      case '/lib1':
        return new NoirPackage('/lib1', '/lib1/src', {
          dependencies: {},
          package: {
            name: 'lib1',
            type: 'lib',
          },
        });

      case '/lib2':
        return new NoirPackage('/lib2', '/lib2/src', {
          dependencies: {
            lib3: {
              path: '/lib3',
            },
          },
          package: {
            name: 'lib2',
            type: 'lib',
          },
        });

      case '/lib3':
        return new NoirPackage('/lib3', '/lib3/src', {
          dependencies: {},
          package: {
            name: 'lib3',
            type: 'lib',
          },
        });

      default:
        throw new Error();
    }
  }
}
