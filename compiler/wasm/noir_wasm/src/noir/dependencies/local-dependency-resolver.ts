import { NoirDependencyConfig } from '../types/noir_package_config';

import { isAbsolute, join } from 'path';
import { IFs } from 'memfs';

import { NoirPackage } from '../package';
import { NoirDependency, NoirDependencyResolver } from './dependency-resolver';

/**
 * Resolves dependencies on-disk, relative to current package
 */
export class LocalDependencyResolver implements NoirDependencyResolver {
  #fm: IFs;

  constructor(fm: IFs) {
    this.#fm = fm;
  }

  resolveDependency(parent: NoirPackage, config: NoirDependencyConfig): Promise<NoirDependency | null> {
    if ('path' in config) {
      const parentPath = parent.getPackagePath();
      const dependencyPath = isAbsolute(config.path) ? config.path : join(parentPath, config.path);
      return Promise.resolve({
        // unknown version, Nargo.toml doesn't have a version field
        version: undefined,
        package: NoirPackage.open(dependencyPath, this.#fm),
      });
    } else {
      return Promise.resolve(null);
    }
  }
}
