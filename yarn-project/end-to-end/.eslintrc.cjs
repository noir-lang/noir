const baseConfig = require('@aztec/foundation/eslint');

const e2eConfig = {
  overrides: [
    {
      files: ['*.ts'],
      rules: {
        'no-restricted-imports': [
          'off',
          {
            name: '@aztec/circuit-types/stats',
          },
        ],
      },
    },
    {
      files: ['*.ts'],
      rules: {
        'no-restricted-imports': [
          'error',
          {
            name: '@aztec/circuit-types',
            message:
              'Please do not import from @aztec/circuit-types directly. Instead, export the required type from @aztec/aztec.js.',
          },
        ],
      },
    },
  ],
};

module.exports = {
  ...baseConfig,
  overrides: [...baseConfig.overrides, ...e2eConfig.overrides],
};
