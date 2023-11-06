import { NoirDependencyConfig } from '@aztec/foundation/noir';

import { NoirPackage } from '../package.js';

/**
 * A Noir dependency
 */
export type NoirDependency = {
  /** version string as determined by the resolver */
  version?: string;
  /** the actual package source code */
  package: NoirPackage;
};

/**
 * Resolves a dependency for a package.
 */
export interface NoirDependencyResolver {
  /**
   * Resolve a dependency for a package.
   * @param pkg - The package to resolve dependencies for
   * @param dep - The dependency config to resolve
   */
  resolveDependency(pkg: NoirPackage, dep: NoirDependencyConfig): Promise<NoirDependency | null>;
}
