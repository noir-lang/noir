require('@rushstack/eslint-patch/modern-module-resolution');

module.exports = {
  extends: ['@aztec/foundation/eslint'],
  parserOptions: { tsconfigRootDir: __dirname },
};
