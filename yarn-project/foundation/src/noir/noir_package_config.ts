import { z } from 'zod';

const noirGitDependencySchema = z.object({
  git: z.string(),
  tag: z.string(),
  directory: z.string().optional(),
});

const noirLocalDependencySchema = z.object({
  path: z.string(),
});

const noirPackageConfigSchema = z.object({
  package: z.object({
    name: z.string().default(''),
    type: z.enum(['lib', 'contract', 'bin']).default('bin'),
    entry: z.string().optional(),
    description: z.string().optional(),
    authors: z.array(z.string()).optional(),
    // eslint-disable-next-line camelcase
    compiler_version: z.string().optional(),
    backend: z.string().optional(),
    license: z.string().optional(),
  }),
  dependencies: z.record(z.union([noirGitDependencySchema, noirLocalDependencySchema])).default({}),
});

/**
 * Noir package configuration.
 */
export type NoirPackageConfig = z.infer<typeof noirPackageConfigSchema>;

/**
 * A remote package dependency.
 */
export type NoirGitDependencyConfig = z.infer<typeof noirGitDependencySchema>;

/**
 * A local package dependency.
 */
export type NoirLocalDependencyConfig = z.infer<typeof noirLocalDependencySchema>;

/**
 * A package dependency.
 */
export type NoirDependencyConfig = NoirGitDependencyConfig | NoirLocalDependencyConfig;

/**
 * Checks that an object is a package configuration.
 * @param config - Config to check
 */
export function parseNoirPackageConfig(config: any): NoirPackageConfig {
  return noirPackageConfigSchema.parse(config);
}
