import { defineConfig, globalIgnores } from 'eslint/config';
import js from '@eslint/js';
import typescriptEslint from '@typescript-eslint/eslint-plugin';
import prettier from 'eslint-plugin-prettier';
import globals from 'globals';
import tsParser from '@typescript-eslint/parser';

export default defineConfig([
  {
    extends: [js.configs.recommended, ...typescriptEslint.configs['flat/recommended']],

    plugins: { prettier },

    languageOptions: {
      globals: { ...globals.browser, ...globals.node },

      parser: tsParser,
    },

    rules: {
      'comma-spacing': ['error', { before: false, after: true }],

      'no-unused-vars': 'off',

      '@typescript-eslint/no-unused-vars': [
        'warn',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' },
      ],

      'prettier/prettier': 'error',
    },
  },
  {
    files: ['**/*.test.ts'],

    rules: { '@typescript-eslint/no-unused-expressions': 'off' },
  },
  globalIgnores([
    'acvm-repo/acvm_js/web/',
    'acvm-repo/acvm_js/nodejs/',
    'compiler/wasm/dist/',
    'compiler/wasm/build/',
    'tooling/noir_codegen/lib/',
    'tooling/noir_js_types/lib/',
    'tooling/noir_js/lib/',
    'tooling/noirc_abi_wasm/web/',
    'tooling/noirc_abi_wasm/nodejs/',
  ]),
]);
