// TODO dont keep this around too long!
// we want to close the jsdoc gap and delete this, moving to .eslintc.cjs which enables jsdoc
const fs = require('fs');

function getFirstExisting(files) {
  for (const file of files) {
    if (fs.existsSync(file)) {
      return file;
    }
  }
  throw new Error('Found no existing file of: ' + files.join(', ') + ' at ' + process.cwd());
}

module.exports = {
  extends: ['eslint:recommended', 'plugin:@typescript-eslint/recommended', 'prettier'],
  root: true,
  parser: '@typescript-eslint/parser',
  // plugins: ['@typescript-eslint', 'eslint-plugin-tsdoc', 'jsdoc'],
  plugins: ['@typescript-eslint'],
  overrides: [
    {
      files: ['*.ts', '*.tsx'],
      parserOptions: {
        // hacky workaround for CI not having the same tsconfig setup
        project: getFirstExisting([
          './tsconfig.eslint.json',
          '../tsconfig.eslint.json',
          __dirname + '/../tsconfig.eslint.json',
          './tsconfig.dest.json',
        ]),
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
    '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
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
