require('@rushstack/eslint-patch/modern-module-resolution');

module.exports = {
  extends: ['@aztec/eslint-config'],
  parserOptions: { tsconfigRootDir: __dirname },
  rules: {
    'tsdoc/syntax': 'off',
    'jsdoc/require-jsdoc': 'off',
    'jsdoc/require-description': 'off',
    'jsdoc/require-description-complete-sentence': 'off',
    'jsdoc/require-hyphen-before-param-description': 'off',
    'jsdoc/require-param': 'off',
    'jsdoc/require-param-description': 'off',
    'jsdoc/require-param-name': 'off',
    'jsdoc/require-property': 'off',
    'jsdoc/require-property-description': 'off',
    'jsdoc/require-property-name': 'off',
    'jsdoc/require-returns': 'off',
    'jsdoc/require-returns-description': 'off',
  },
};
