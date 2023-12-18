import { isAbsolute, join } from 'path';

import { FileManager } from '../file-manager/file-manager';
import { NoirPackage } from '../package';
import { NoirDependency, NoirDependencyResolver } from './dependency-resolver';
import { NoirDependencyConfig } from '../../types/noir_package_config';

/**
 * Resolves dependencies on-disk, relative to current package
 */
export class LocalDependencyResolver implements NoirDependencyResolver {
  #fm: FileManager;

  constructor(fm: FileManager) {
    this.#fm = fm;
  }

  async resolveDependency(parent: NoirPackage, config: NoirDependencyConfig): Promise<NoirDependency | null> {
    if ('path' in config) {
      const parentPath = parent.getPackagePath();
      const dependencyPath = isAbsolute(config.path) ? config.path : join(parentPath, config.path);
      return {
        // unknown version, Nargo.toml doesn't have a version field
        version: undefined,
        package: await NoirPackage.open(dependencyPath, this.#fm),
      };
    } else {
      return null;
    }
  }
}
