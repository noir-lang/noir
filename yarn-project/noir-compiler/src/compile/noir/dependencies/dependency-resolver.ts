import { NoirDependencyConfig } from '@aztec/foundation/noir';

import { NoirPackage } from '../package.js';

/**
 * Resolves a dependency for a package.
 */
export interface DependencyResolver {
  /**
   * Resolve a dependency for a package.
   * @param pkg - The package to resolve dependencies for
   * @param dep - The dependency config to resolve
   */
  resolveDependency(pkg: NoirPackage, dep: NoirDependencyConfig): Promise<NoirPackage | null>;
}
