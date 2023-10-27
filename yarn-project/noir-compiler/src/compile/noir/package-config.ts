import { z } from 'zod';

const noirGitDependency = z.object({
  git: z.string(),
  tag: z.string().optional(),
  directory: z.string().optional(),
});

const noirLocalDependency = z.object({
  path: z.string(),
});

const noirPackageConfig = z.object({
  package: z.object({
    name: z.string(),
    type: z.enum(['lib', 'contract', 'binary']),
  }),
  dependencies: z.record(z.union([noirGitDependency, noirLocalDependency])),
});

/**
 * Noir package configuration.
 */
export type NoirPackageConfig = z.infer<typeof noirPackageConfig>;

/**
 * A remote package dependency.
 */
export type NoirGitDependencyConfig = z.infer<typeof noirGitDependency>;

/**
 * A local package dependency.
 */
export type NoirLocalDependencyConfig = z.infer<typeof noirLocalDependency>;

/**
 * A package dependency.
 */
export type NoirDependencyConfig = NoirGitDependencyConfig | NoirLocalDependencyConfig;

/**
 * Checks that an object is a package configuration.
 * @param config - Config to check
 */
export function parsePackageConfig(config: any): NoirPackageConfig {
  return noirPackageConfig.parse(config);
}
