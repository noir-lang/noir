type NoirGitDependencySchema = {
  git: string;
  tag: string;
  directory?: string;
};

type NoirLocalDependencySchema = {
  path: string;
};

type NoirPackageType = 'lib' | 'contract' | 'bin';
type NoirPackageConfigSchema = {
  package: {
    name: string;
    type: NoirPackageType;
    entry?: string;
    description?: string;
    authors?: string[];
    compiler_version?: string;
    backend?: string;
    license?: string;
  };
  dependencies?: Record<string, GitDependencyConfig | LocalDependencyConfig>;
};

/**
 * Noir package configuration.
 */
export type PackageConfig = NoirPackageConfigSchema;

/**
 * A remote package dependency.
 */
export type GitDependencyConfig = NoirGitDependencySchema;

/**
 * A local package dependency.
 */
export type LocalDependencyConfig = NoirLocalDependencySchema;

/**
 * A package dependency.
 */
export type DependencyConfig = GitDependencyConfig | LocalDependencyConfig;

/**
 * Checks that an object is a package configuration.
 * @param config - Config to check
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function parseNoirPackageConfig(config: any): PackageConfig {
  return config;
}
