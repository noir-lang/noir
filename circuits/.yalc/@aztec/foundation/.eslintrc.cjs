module.exports = {
  extends: ['eslint:recommended', 'plugin:@typescript-eslint/recommended', 'prettier'],
  root: true,
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint'],
  overrides: [
    {
      files: ['*.ts', '*.tsx'],
      parserOptions: {
        project: ['./tsconfig.json'],
      },
    },
  ],
  env: {
    node: true,
  },
  rules: {
    '@typescript-eslint/explicit-module-boundary-types': 'off',
    '@typescript-eslint/no-non-null-assertion': 'off',
    '@typescript-eslint/no-explicit-any': 'off',
    '@typescript-eslint/no-empty-function': 'off',
    '@typescript-eslint/await-thenable': 'error',
    '@typescript-eslint/no-floating-promises': 2,
    'require-await': 2,
    'no-constant-condition': 'off',
    camelcase: 2,
    'no-restricted-imports': [
      'warn',
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
  },
  ignorePatterns: ['node_modules', 'dest*', 'dist', '*.js', '.eslintrc.cjs'],
};
