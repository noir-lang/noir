import { isAbsolute, join } from 'path';

import { FileManager } from '../file-manager/file-manager';
import { Package } from '../package';
import { Dependency, DependencyResolver } from './dependency-resolver';
import { DependencyConfig } from '../../types/noir_package_config';

/**
 * Resolves dependencies on-disk, relative to current package
 */
export class LocalDependencyResolver implements DependencyResolver {
  #fm: FileManager;

  constructor(fm: FileManager) {
    this.#fm = fm;
  }

  async resolveDependency(parent: Package, config: DependencyConfig): Promise<Dependency | null> {
    if ('path' in config) {
      const parentPath = parent.getPackagePath();
      const dependencyPath = isAbsolute(config.path) ? config.path : join(parentPath, config.path);
      return {
        // unknown version, Nargo.toml doesn't have a version field
        version: undefined,
        package: await Package.open(dependencyPath, this.#fm),
      };
    } else {
      return null;
    }
  }
}
