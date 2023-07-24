const contexts = [
  'TSMethodDefinition',
  'MethodDefinition',
  'TSParameterProperty[accessibility=public]',
  'TSPropertySignature',
  'PropertySignature',
  'TSInterfaceDeclaration',
  'InterfaceDeclaration',
  'TSPropertyDefinition[accessibility=public]',
  'PropertyDefinition[accessibility=public]',
  'TSTypeAliasDeclaration',
  'TypeAliasDeclaration',
  'TSTypeDeclaration',
  'TypeDeclaration',
  'TSEnumDeclaration',
  'EnumDeclaration',
  'TSClassDeclaration',
  'ClassDeclaration',
  'TSClassExpression',
  'ClassExpression',
  'TSFunctionExpression',
  'FunctionExpression',
  'TSInterfaceExpression',
  'InterfaceExpression',
  'TSEnumExpression',
  'EnumExpression',
];

let tsconfigPaths;
if (process.env.DOCKER_ENV) {
  tsconfigPaths = ['./tsconfig.node.json', './tsconfig.browser.json'];
} else {
  tsconfigPaths = ['./ts/tsconfig.node.json', './ts/tsconfig.browser.json'];
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
        project: tsconfigPaths,
      },
    },
    {
      files: ['*.test.ts', '*.test.tsx'],
      // parserOptions: {
      //   tsconfigRootDir: __dirname + '/..',
      //   project: __dirname + '/../tsconfig.test.json',
      // },
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
    '@typescript-eslint/no-unused-vars': 'warn',
    'require-await': 2,
    'no-constant-condition': 'off',
    camelcase: 2,
    'no-restricted-imports': [
      'warn',
      {
        patterns: [
          {
            group: ['dest'],
            message: 'You should not be importing from a build directory. Did you accidentally do a relative import?',
          },
        ],
      },
    ],
    // 'tsdoc/syntax': 'warn',
    // 'jsdoc/require-jsdoc': [
    //   'warn',
    //   {
    //     contexts,
    //     checkConstructors: false,
    //     checkGetters: true,
    //     checkSetters: true,
    //   },
    // ],
    // 'jsdoc/require-description': ['warn', { contexts }],
    // 'jsdoc/require-description-complete-sentence': ['warn'],
    // 'jsdoc/require-hyphen-before-param-description': ['warn'],
    // 'jsdoc/require-param': ['warn', { contexts, checkDestructured: false }],
    // 'jsdoc/require-param-description': ['warn', { contexts }],
    // 'jsdoc/require-param-name': ['warn', { contexts }],
    // 'jsdoc/require-property': ['warn', { contexts }],
    // 'jsdoc/require-property-description': ['warn', { contexts }],
    // 'jsdoc/require-property-name': ['warn', { contexts }],
    // 'jsdoc/require-returns': ['warn', { contexts }],
    // 'jsdoc/require-returns-description': ['warn', { contexts }],
  },
  ignorePatterns: ['node_modules', 'dest*', 'dist', '*.js', '.eslintrc.cjs'],
};
