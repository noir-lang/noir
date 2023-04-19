// Unknown Error
export const UNKNOWN_ERROR = 'UNKNOWN_ERROR';

// Invalid argument (e.g. value is incompatible with type) to a function:
//   - arg: The argument name that was invalid
//   - value: The value of the argument
export const INVALID_ARGUMENT = 'INVALID_ARGUMENT';

// Missing argument to a function:
//   - count: The number of arguments received
//   - expectedCount: The number of arguments expected
export const MISSING_ARGUMENT = 'MISSING_ARGUMENT';

// Too many arguments
//   - count: The number of arguments received
//   - expectedCount: The number of arguments expected
export const UNEXPECTED_ARGUMENT = 'UNEXPECTED_ARGUMENT';

const _censorErrors = false;

/**
 * Throws a detailed error with a custom message, code, and additional information.
 * The error message can be censored by setting the '_censorErrors' variable to true.
 * In that case, a generic 'unknown error' message will be thrown instead of the custom message.
 *
 * @param message - The custom error message to display.
 * @param code - The specific error code for this error (default is UNKNOWN_ERROR).
 * @param params - An object containing additional information related to the error.
 * @returns - This function always throws an error and does not return any value.
 */
export function throwError(message: string, code: string = UNKNOWN_ERROR, params: any = {}): never {
  if (_censorErrors) {
    throw new Error('unknown error');
  }

  const messageDetails: Array<string> = [];
  Object.keys(params).forEach(key => {
    try {
      messageDetails.push(key + '=' + JSON.stringify(params[key]));
    } catch (error) {
      messageDetails.push(key + '=' + JSON.stringify(params[key].toString()));
    }
  });
  messageDetails.push('version=1');

  const reason = message;
  if (messageDetails.length) {
    message += ' (' + messageDetails.join(', ') + ')';
  }

  // @TODO: Any??
  const error: any = new Error(message);
  error.reason = reason;
  error.code = code;

  Object.keys(params).forEach(function (key) {
    error[key] = params[key];
  });

  throw error;
}

/**
 * Validates the number of arguments provided against the expected count and throws an error if they do not match.
 * This function is useful for checking the right number of arguments are passed to a function, especially in cases
 * where optional arguments are involved. It appends a custom message suffix when provided.
 *
 * @param count - The actual number of arguments received by the function.
 * @param expectedCount - The expected number of arguments for the function.
 * @param suffix - Optional string to be appended to the error message when thrown.
 * @throws  If either too few or too many arguments are provided.
 */
export function checkArgumentCount(count: number, expectedCount: number, suffix?: string): void {
  if (!suffix) {
    suffix = '';
  }
  if (count < expectedCount) {
    throwError('missing argument' + suffix, MISSING_ARGUMENT, { count: count, expectedCount: expectedCount });
  }
  if (count > expectedCount) {
    throwError('too many arguments' + suffix, UNEXPECTED_ARGUMENT, { count: count, expectedCount: expectedCount });
  }
}
