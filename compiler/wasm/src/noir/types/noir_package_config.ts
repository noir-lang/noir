type noirGitDependencySchema = {
  git: string;
  tag: string;
  directory?: string;
};

type noirLocalDependencySchema = {
  path: string;
};

enum type {
  lib = 'lib',
  contract = 'contract',
  bin = 'bin',
}

type noirPackageConfigSchema = {
  package: {
    name: string;
    type: type;
    entry: string;
    description: string;
    authors: string[];
    compiler_version: string;
    backend: string;
    license: string;
  };
  dependencies: Record<string, NoirGitDependencyConfig | NoirLocalDependencyConfig>;
};

/**
 * Noir package configuration.
 */
export type NoirPackageConfig = noirPackageConfigSchema;

/**
 * A remote package dependency.
 */
export type NoirGitDependencyConfig = noirGitDependencySchema;

/**
 * A local package dependency.
 */
export type NoirLocalDependencyConfig = noirLocalDependencySchema;

/**
 * A package dependency.
 */
export type NoirDependencyConfig = NoirGitDependencyConfig | NoirLocalDependencyConfig;

/**
 * Checks that an object is a package configuration.
 * @param config - Config to check
 */
export function parseNoirPackageConfig(config: any): NoirPackageConfig {
  return config;
}
