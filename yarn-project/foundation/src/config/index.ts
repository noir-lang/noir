import { type EnvVar } from './env_var.js';

export { EnvVar } from './env_var.js';

export interface ConfigMapping {
  env?: EnvVar;
  parseEnv?: (val: string) => any;
  defaultValue?: any;
  printDefault?: (val: any) => string;
  description: string;
  isBoolean?: boolean;
}

export function isBooleanConfigValue<T>(obj: T, key: keyof T): boolean {
  return typeof obj[key] === 'boolean';
}

export type ConfigMappingsType<T> = Record<keyof T, ConfigMapping>;

export function getConfigFromMappings<T>(configMappings: ConfigMappingsType<T>): T {
  const config = {} as T;

  for (const key in configMappings) {
    if (configMappings[key]) {
      const { env, parseEnv, defaultValue: def } = configMappings[key];
      // Special case for L1 contract addresses which is an object of config values
      if (key === 'l1Contracts' && def) {
        (config as any)[key] = getConfigFromMappings(def);
      } else {
        const val = env ? process.env[env] : undefined;
        if (val !== undefined) {
          (config as any)[key] = parseEnv ? parseEnv(val) : val;
        } else if (def !== undefined) {
          (config as any)[key] = def;
        }
      }
    }
  }

  return config;
}

/**
 * Filters out a service's config mappings to exclude certain keys.
 * @param configMappings - The service's config mappings
 * @param keysToFilter - The keys to filter out
 * @returns The filtered config mappings
 */
export function filterConfigMappings<T, K extends keyof T>(
  configMappings: ConfigMappingsType<T>,
  keysToFilter: K[],
): ConfigMappingsType<Omit<T, K>> {
  return Object.fromEntries(
    Object.entries(configMappings).filter(([key]) => !keysToFilter.includes(key as K)),
  ) as ConfigMappingsType<Omit<T, K>>;
}

/**
 * Generates parseEnv and default values for a numerical config value.
 * @param defaultVal - The default numerical value to use if the environment variable is not set or is invalid
 * @returns Object with parseEnv and default values for a numerical config value
 */
export function numberConfigHelper(defaultVal: number): Pick<ConfigMapping, 'parseEnv' | 'defaultValue'> {
  return {
    parseEnv: (val: string) => safeParseNumber(val, defaultVal),
    defaultValue: defaultVal,
  };
}

/**
 * Generates parseEnv and default values for a boolean config value.
 * @param defaultVal - The default value to use if the environment variable is not set or is invalid
 * @returns Object with parseEnv and default values for a boolean config value
 */
export function booleanConfigHelper(
  defaultVal = false,
): Required<Pick<ConfigMapping, 'parseEnv' | 'defaultValue' | 'isBoolean'> & { parseVal: (val: string) => boolean }> {
  const parse = (val: string | boolean) => (typeof val === 'boolean' ? val : ['1', 'true', 'TRUE'].includes(val));
  return {
    parseEnv: parse,
    parseVal: parse,
    defaultValue: defaultVal,
    isBoolean: true,
  };
}

/**
 * Safely parses a number from a string.
 * If the value is not a number or is not a safe integer, the default value is returned.
 * @param value - The string value to parse
 * @param defaultValue - The default value to return
 * @returns Either parsed value or default value
 */
function safeParseNumber(value: string, defaultValue: number): number {
  const parsedValue = parseInt(value, 10);
  return Number.isSafeInteger(parsedValue) ? parsedValue : defaultValue;
}

/**
 * Picks specific keys from the given configuration mappings.
 *
 * @template T - The type of the full configuration object.
 * @template K - The keys to pick from the configuration object.
 * @param {ConfigMappingsType<T>} configMappings - The full configuration mappings object.
 * @param {K[]} keys - The keys to pick from the configuration mappings.
 * @returns {ConfigMappingsType<Pick<T, K>>} - A new configuration mappings object containing only the specified keys.
 */
export function pickConfigMappings<T, K extends keyof T>(
  configMappings: ConfigMappingsType<T>,
  keys: K[],
): ConfigMappingsType<Pick<T, K>> {
  return Object.fromEntries(keys.map(key => [key, configMappings[key]])) as ConfigMappingsType<Pick<T, K>>;
}

/**
 * Extracts the default configuration values from the given configuration mappings.
 *
 * @template T - The type of the configuration object.
 * @param {ConfigMappingsType<T>} configMappings - The configuration mappings object.
 * @returns {T} - The configuration object with default values.
 */
export function getDefaultConfig<T>(configMappings: ConfigMappingsType<T>): T {
  const defaultConfig = {} as T;

  for (const key in configMappings) {
    if (configMappings[key] && configMappings[key].defaultValue !== undefined) {
      (defaultConfig as any)[key] = configMappings[key].defaultValue;
    }
  }

  return defaultConfig;
}
