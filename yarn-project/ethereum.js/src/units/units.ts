/**
 * Converts the value to a decimal string representation with the given precision.
 * The digits outside the precision are simply discarded (i.e. the result is floored).
 * This ensures we never report more funds than actually exists.
 * Trailing 0's are also removed.
 * @param value to convert to string
 * @param decimals the number of least significant digits of value that represent the decimal
 * @param precision the number of decimal places to return
 */
export function fromBaseUnits(value: bigint, decimals: number, precision: number = decimals) {
  const neg = value < BigInt(0);
  const valStr = value
    .toString()
    .slice(neg ? 1 : 0)
    .padStart(decimals + 1, '0');
  const integer = valStr.slice(0, valStr.length - decimals);
  const fractionalTrim = valStr.slice(-decimals);
  let end = fractionalTrim.length - 1;
  while (fractionalTrim[end] === '0') --end;
  const fractional = fractionalTrim.slice(0, end + 1);
  return (neg ? '-' : '') + (fractional ? `${integer}.${fractional.slice(0, precision)}` : integer);
}

/**
 * Converts the value from a decimal string to bigint value.
 * @param valueString to convert to bigint
 * @param decimals the number of least significant digits of value that represent the decimal
 */
export function toBaseUnits(valueString: string, decimals: number) {
  const [integer, decimal] = valueString.split('.');
  const fractional = (decimal || '').replace(/0+$/, '').slice(0, decimals);
  const scalingFactor = BigInt(10) ** BigInt(decimals);
  const fractionalScale = scalingFactor / BigInt(10) ** BigInt(fractional.length || 0);
  return BigInt(fractional || 0) * fractionalScale + BigInt(integer || 0) * scalingFactor;
}
