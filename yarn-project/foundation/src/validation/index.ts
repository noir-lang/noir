/** Utility function to throw an error if a required value is missing. */
export function required<T>(value: T | undefined, errMsg?: string): T {
  if (value === undefined) {
    throw new Error(errMsg || 'Value is required');
  }
  return value;
}
