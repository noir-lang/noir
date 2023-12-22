type noirGitDependencySchema = {
  git: string;
  tag: string;
  directory?: string;
};

type noirLocalDependencySchema = {
  path: string;
};

type noirPackageType = 'lib' | 'contract' | 'bin';
type noirPackageConfigSchema = {
  package: {
    name: string;
    type: noirPackageType;
    entry?: string;
    description?: string;
    authors?: string[];
    compiler_version?: string;
    backend?: string;
    license?: string;
  };
  dependencies: Record<string, GitDependencyConfig | LocalDependencyConfig>;
};

/**
 * Noir package configuration.
 */
export type PackageConfig = noirPackageConfigSchema;

/**
 * A remote package dependency.
 */
export type GitDependencyConfig = noirGitDependencySchema;

/**
 * A local package dependency.
 */
export type LocalDependencyConfig = noirLocalDependencySchema;

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
