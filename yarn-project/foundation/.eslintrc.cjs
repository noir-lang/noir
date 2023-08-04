// See https://typescript-eslint.io/play/#ts=5.1.6&showAST=es&fileType=.ts
const contexts = [
  // All methods in an interface
  'TSInterfaceDeclaration TSMethodSignature',
  // All public methods in a class that does not implement an interface
  'ClassDeclaration[implements.length=0] MethodDefinition[accessibility=public]',
  // TODO: All methods public by default in a class that does not implement an interface
  // 'ClassDeclaration[implements.length=0] MethodDefinition[accessibility=undefined][key.type=Identifier]',
  // TODO: All export const from the top level of a file
  // 'ExportNamedDeclaration[declaration.type=VariableDeclaration]',
  // Legacy contexts (needs review)
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

const JSDOC_RULES_LEVEL = 'error';

module.exports = {
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
  root: true,
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint', 'eslint-plugin-tsdoc', 'jsdoc'],
  overrides: [
    {
      files: ['*.ts', '*.tsx'],
      parserOptions: {
        // hacky workaround for CI not having the same tsconfig setup
        project: true,
      },
    },
    {
      files: '*.test.ts',
      rules: {
        'jsdoc/require-jsdoc': 'off',
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
    '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
    'require-await': 2,
    'no-console': 'error',
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
    'import/no-unresolved': [
      'error',
      {
        ignore: [
          // See https://github.com/import-js/eslint-plugin-import/issues/2703
          '@libp2p/bootstrap',
          // Seems like ignoring l1-artifacts in the eslint call messes up no-unresolved
          '@aztec/l1-artifacts',
        ],
      },
    ],
    'import/no-extraneous-dependencies': 'error',
    'tsdoc/syntax': JSDOC_RULES_LEVEL,
    'jsdoc/require-jsdoc': [
      JSDOC_RULES_LEVEL,
      {
        contexts,
        checkConstructors: false,
        checkGetters: true,
        checkSetters: true,
      },
    ],
    'jsdoc/require-description': [JSDOC_RULES_LEVEL, { contexts }],
    'jsdoc/require-hyphen-before-param-description': [JSDOC_RULES_LEVEL],
    'jsdoc/require-param': [JSDOC_RULES_LEVEL, { contexts, checkDestructured: false }],
    'jsdoc/require-param-description': [JSDOC_RULES_LEVEL, { contexts }],
    'jsdoc/require-param-name': [JSDOC_RULES_LEVEL, { contexts }],
    'jsdoc/require-property': [JSDOC_RULES_LEVEL, { contexts }],
    'jsdoc/require-property-description': [JSDOC_RULES_LEVEL, { contexts }],
    'jsdoc/require-property-name': [JSDOC_RULES_LEVEL, { contexts }],
    'jsdoc/require-returns': 'off',
  },
  ignorePatterns: ['node_modules', 'dest*', 'dist', '*.js', '.eslintrc.cjs'],
};
