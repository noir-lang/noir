require("@rushstack/eslint-patch/modern-module-resolution");

module.exports = {
  extends: ["@aztec/eslint-config"],
  parserOptions: { tsconfigRootDir: __dirname },
};
