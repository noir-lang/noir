export const MNEMONIC = 'test test test test test test test test test test test junk';
export const privateKey = Buffer.from('ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80', 'hex');
export const privateKey2 = Buffer.from('59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d', 'hex');

/// Common errors
export const U128_UNDERFLOW_ERROR = "Assertion failed: attempt to subtract with underflow 'hi == high'";
export const U128_OVERFLOW_ERROR = "Assertion failed: attempt to add with overflow 'hi == high'";
export const BITSIZE_TOO_BIG_ERROR =
  "Assertion failed: call to assert_max_bit_size 'self.__assert_max_bit_size(bit_size)'";
