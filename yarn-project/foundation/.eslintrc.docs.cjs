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
  // Legacy contexts below (needs review)
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

const base = require('./.eslintrc.cjs');
const JSDOC_RULES_LEVEL = 'error';

module.exports = {
  ...base,
  plugins: [...base.plugins, 'jsdoc'],
  overrides: [...base.overrides, { files: '*.test.ts', rules: { 'jsdoc/require-jsdoc': 'off' } }],
  rules: {
    ...base.rules,
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
};
