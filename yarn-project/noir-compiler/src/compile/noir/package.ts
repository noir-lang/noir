import { join } from 'node:path';
import { parse as parseToml } from 'toml';

import { FileManager } from './file-manager/file-manager.js';
import {
  NoirGitDependencyConfig,
  NoirLocalDependencyConfig,
  NoirPackageConfig,
  parsePackageConfig,
} from './package-config.js';

const CONFIG_FILE_NAME = 'Nargo.toml';

/**
 * A Noir package.
 */
export class NoirPackage {
  #packagePath: string;
  #srcPath: string;
  #config: NoirPackageConfig;

  public constructor(path: string, srcDir: string, config: NoirPackageConfig) {
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
        entrypoint = 'lib.nr';
        break;
      case 'contract':
      case 'binary':
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
  public getDependencies(): Record<string, NoirGitDependencyConfig | NoirLocalDependencyConfig> {
    return this.#config.dependencies;
  }

  /**
   * Opens a path on the filesystem.
   * @param path - Path to the package.
   * @param fm - A file manager to use.
   * @returns The Noir package at the given location
   */
  public static open(path: string, fm: FileManager): NoirPackage {
    const fileContents = fm.readFileSync(join(path, CONFIG_FILE_NAME), 'utf-8');
    const config = parsePackageConfig(parseToml(fileContents));

    return new NoirPackage(path, join(path, 'src'), config);
  }
}
