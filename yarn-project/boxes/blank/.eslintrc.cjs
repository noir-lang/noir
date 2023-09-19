module.exports = {
  root: true,
  env: { browser: true, es2020: true },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:import/recommended',
    'plugin:import/typescript',
    'prettier',
  ],
  settings: {
    'import/resolver': {
      typescript: true,
      node: true,
    },
  },
  ignorePatterns: ['dist', '.eslintrc.cjs'],
  parser: '@typescript-eslint/parser',
  plugins: [],
  overrides: [
    {
      files: ['*.ts', '*.tsx'],
      parserOptions: {
        // hacky workaround for CI not having the same tsconfig setup
        project: true,
      },
    },
  ],
  rules: {
    '@typescript-eslint/explicit-module-boundary-types': 'off',
    '@typescript-eslint/no-non-null-assertion': 'off',
    '@typescript-eslint/no-explicit-any': 'off',
    '@typescript-eslint/no-empty-function': 'off',
    '@typescript-eslint/await-thenable': 'error',
    '@typescript-eslint/no-floating-promises': 2,
    '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
    'require-await': 2,
    'no-console': 'warn',
    'no-constant-condition': 'off',
    camelcase: 2,
    'no-restricted-imports': [
      'error',
      {
        patterns: [
          {
            group: ['client-dest'],
            message: "Fix this absolute garbage import. It's your duty to solve it before it spreads.",
          },
          {
            group: ['dest'],
            message: 'You should not be importing from a build directory. Did you accidentally do a relative import?',
          },
        ],
      },
    ],
    'import/no-unresolved': 'error',
    'import/no-extraneous-dependencies': 'error',
  },
};
