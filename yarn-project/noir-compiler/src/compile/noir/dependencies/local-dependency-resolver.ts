import { NoirDependencyConfig } from '@aztec/foundation/noir';

import { resolve } from 'path';

import { FileManager } from '../file-manager/file-manager.js';
import { NoirPackage } from '../package.js';
import { NoirDependency, NoirDependencyResolver } from './dependency-resolver.js';

/**
 * Resolves dependencies on-disk, relative to current package
 */
export class LocalDependencyResolver implements NoirDependencyResolver {
  #fm: FileManager;

  constructor(fm: FileManager) {
    this.#fm = fm;
  }

  resolveDependency(pkg: NoirPackage, config: NoirDependencyConfig): Promise<NoirDependency | null> {
    if ('path' in config) {
      return Promise.resolve({
        // unknown version, Nargo.toml doesn't have a version field
        version: undefined,
        package: NoirPackage.open(resolve(pkg.getPackagePath(), config.path), this.#fm),
      });
    } else {
      return Promise.resolve(null);
    }
  }
}
