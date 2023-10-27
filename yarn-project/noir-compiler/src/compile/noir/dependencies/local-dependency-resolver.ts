import { resolve } from 'path';

import { FileManager } from '../file-manager/file-manager.js';
import { NoirDependencyConfig } from '../package-config.js';
import { NoirPackage } from '../package.js';
import { DependencyResolver } from './dependency-resolver.js';

/**
 * Resolves dependencies on-disk, relative to current package
 */
export class LocalDependencyResolver implements DependencyResolver {
  #fm: FileManager;

  constructor(fm: FileManager) {
    this.#fm = fm;
  }

  resolveDependency(pkg: NoirPackage, config: NoirDependencyConfig): Promise<NoirPackage | null> {
    if ('path' in config) {
      return Promise.resolve(NoirPackage.open(resolve(pkg.getPackagePath(), config.path), this.#fm));
    } else {
      return Promise.resolve(null);
    }
  }
}
