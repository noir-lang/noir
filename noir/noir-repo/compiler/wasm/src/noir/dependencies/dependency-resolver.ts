import { DependencyConfig } from '../../types/noir_package_config';
import { Package } from '../package';

/**
 * A Noir dependency
 */
export type Dependency = {
  /** version string as determined by the resolver */
  version?: string;
  /** the actual package source code */
  package: Package;
};

/**
 * Resolves a dependency for a package.
 */
export interface DependencyResolver {
  /**
   * Resolve a dependency for a package.
   * @param pkg - The package to resolve dependencies for
   * @param dep - The dependency config to resolve
   */
  resolveDependency(pkg: Package, dep: DependencyConfig): Promise<Dependency | null>;
}
