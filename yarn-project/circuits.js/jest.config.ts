import type { Config } from 'jest';

const config: Config = {
  preset: 'ts-jest/presets/default-esm',
  moduleNameMapper: {
    '^(\\.{1,2}/.*)\\.[cm]?js$': '$1',
  },
  testRegex: './src/.*\\.test\\.(js|mjs|ts)$',
  rootDir: './src',
};

export default config;
