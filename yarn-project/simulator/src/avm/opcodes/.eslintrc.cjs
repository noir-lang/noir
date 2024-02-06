const baseConfig = require('@aztec/foundation/eslint');
module.exports = {
  ...baseConfig,
  rules: {
    ...baseConfig.rules,
    'require-await': 'off',
  },
};
