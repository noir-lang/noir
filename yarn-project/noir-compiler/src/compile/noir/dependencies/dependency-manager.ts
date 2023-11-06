import { LogFn, createDebugOnlyLogger } from '@aztec/foundation/log';
import { NoirDependencyConfig } from '@aztec/foundation/noir';

import { join } from 'node:path';

import { NoirPackage } from '../package.js';
import { NoirDependency, NoirDependencyResolver } from './dependency-resolver.js';

/**
 * Noir Dependency Resolver
 */
export class NoirDependencyManager {
  #entryPoint: NoirPackage;
  #libraries = new Map<string, NoirDependency>();
  #dependencies = new Map<string, string[]>();
  #log: LogFn;
  #resolvers: readonly NoirDependencyResolver[];

  /**
   * Creates a new dependency resolver
   * @param resolvers - A list of dependency resolvers to use
   * @param entryPoint - The entry point of the project
   */
  constructor(resolvers: readonly NoirDependencyResolver[] = [], entryPoint: NoirPackage) {
    this.#log = createDebugOnlyLogger('noir:dependency-resolver');
    this.#resolvers = resolvers;
    this.#entryPoint = entryPoint;
  }

  /**
   * Gets dependencies for the entry point
   */
  public getEntrypointDependencies() {
    return this.#dependencies.get('') ?? [];
  }

  /**
   * A map of library dependencies
   */
  public getLibraryDependencies() {
    const entries = Array.from(this.#dependencies.entries());
    return Object.fromEntries(entries.filter(([name]) => name !== ''));
  }

  /**
   * Resolves dependencies for a package.
   */
  public async resolveDependencies(): Promise<void> {
    await this.#recursivelyResolveDependencies('', this.#entryPoint);
  }

  /**
   * Gets the version of a dependency in the dependency tree
   * @param name - Dependency name
   * @returns The dependency's version
   */
  public getVersionOf(name: string): string | undefined {
    const dep = this.#libraries.get(name);
    return dep?.version;
  }

  async #recursivelyResolveDependencies(packageName: string, noirPackage: NoirPackage): Promise<void> {
    for (const [name, config] of Object.entries(noirPackage.getDependencies())) {
      // TODO what happens if more than one package has the same name but different versions?
      if (this.#libraries.has(name)) {
        this.#log(`skipping already resolved dependency ${name}`);
        this.#dependencies.set(packageName, [...(this.#dependencies.get(packageName) ?? []), name]);

        continue;
      }

      const dependency = await this.#resolveDependency(noirPackage, config);
      if (dependency.package.getType() !== 'lib') {
        this.#log(`Non-library package ${name}`, config);
        throw new Error(`Dependency ${name} is not a library`);
      }

      this.#libraries.set(name, dependency);
      this.#dependencies.set(packageName, [...(this.#dependencies.get(packageName) ?? []), name]);

      await this.#recursivelyResolveDependencies(name, dependency.package);
    }
  }

  async #resolveDependency(pkg: NoirPackage, config: NoirDependencyConfig): Promise<NoirDependency> {
    let dependency: NoirDependency | null = null;
    for (const resolver of this.#resolvers) {
      dependency = await resolver.resolveDependency(pkg, config);
      if (dependency) {
        break;
      }
    }

    if (!dependency) {
      throw new Error('Dependency not resolved');
    }

    return dependency;
  }

  /**
   * Gets the names of the crates in this dependency list
   */
  public getPackageNames() {
    return [...this.#libraries.keys()];
  }

  /**
   * Looks up a dependency
   * @param sourceId - The source being resolved
   * @returns The path to the resolved file
   */
  public findFile(sourceId: string): string | null {
    const [lib, ...path] = sourceId.split('/').filter(x => x);
    const dep = this.#libraries.get(lib);
    if (dep) {
      return join(dep.package.getSrcPath(), ...path);
    } else {
      return null;
    }
  }
}
