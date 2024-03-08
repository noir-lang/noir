import { parse } from '@ltd/j-toml';
import { join } from 'path';

import { FileManager } from './file-manager/file-manager';
import { DependencyConfig, PackageConfig, parseNoirPackageConfig } from '../types/noir_package_config';

const CONFIG_FILE_NAME = 'Nargo.toml';
const SOURCE_EXTENSIONS = ['.nr'];

/**
 * An array of sources for a package
 */
type SourceList = Array<{
  /**
   * The source path, taking into account modules and aliases. Eg: mylib/mod/mysource.nr
   */
  path: string;
  /**
   * Resolved source plaintext
   */
  source: string;
}>;

/**
 * A Noir package.
 */
export class Package {
  #packagePath: string;
  #srcPath: string;
  #config: PackageConfig;
  #version: string | null = null;

  public constructor(path: string, srcDir: string, config: PackageConfig) {
    this.#packagePath = path;
    this.#srcPath = srcDir;
    this.#config = config;
  }

  /**
   * Gets this package's path.
   */
  public getPackagePath() {
    return this.#packagePath;
  }

  /**
   * Gets this package's Nargo.toml (NoirPackage)Config.
   */
  public getPackageConfig() {
    return this.#config;
  }

  /**
   * The path to the source directory.
   */
  public getSrcPath() {
    return this.#srcPath;
  }

  /**
   * Gets the entrypoint path for this package.
   */
  public getEntryPointPath(): string {
    let entrypoint: string;

    switch (this.getType()) {
      case 'lib':
        // we shouldn't need to compile `lib` type, since the .nr source is read directly
        // when the lib is used as a dependency elsewhere.
        entrypoint = 'lib.nr';
        break;
      case 'contract':
      case 'bin':
        entrypoint = 'main.nr';
        break;
      default:
        throw new Error(`Unknown package type: ${this.getType()}`);
    }
    // TODO check that `src` exists
    return join(this.#srcPath, entrypoint);
  }

  /**
   * Gets the project type
   */
  public getType() {
    return this.#config.package.type;
  }

  /**
   * Gets this package's dependencies.
   */
  public getDependencies(): Record<string, DependencyConfig> {
    return this.#config.dependencies ?? {};
  }

  /**
   * Gets this package's sources.
   * @param fm - A file manager to use
   * @param alias - An alias for the sources, if this package is a dependency
   */
  public async getSources(fm: FileManager, alias?: string): Promise<SourceList> {
    const handles = await fm.readdir(this.#srcPath, { recursive: true });
    return Promise.all(
      handles
        .filter((handle) => SOURCE_EXTENSIONS.find((ext) => handle.endsWith(ext)))
        .map(async (file) => {
          // Github deps are directly added to the file manager, which causes them to be missing the absolute path to the source file
          // and only include the extraction directory relative to the fm root directory
          // This regexp ensures we remove the "real" source path for all dependencies, providing the compiler with what it expects for each source file:
          // <absoluteSourcePath> -> <sourceAsString> for bin/contract packages
          // <depAlias/relativePathToSource> -> <sourceAsString> for libs
          const suffix = file.replace(new RegExp(`.*${this.#srcPath}`), '');
          return {
            path: this.getType() === 'lib' ? `${alias ? alias : this.#config.package.name}${suffix}` : file,
            source: (await fm.readFile(file, 'utf-8')).toString(),
          };
        }),
    );
  }

  /**
   * Opens a path on the filesystem.
   * @param path - Path to the package.
   * @param fm - A file manager to use.
   * @returns The Noir package at the given location
   */
  public static async open(path: string, fm: FileManager): Promise<Package> {
    const fileContents = await fm.readFile(join(path, CONFIG_FILE_NAME), 'utf-8');
    const config = parseNoirPackageConfig(parse(fileContents));

    return new Package(path, join(path, 'src'), config);
  }
}
