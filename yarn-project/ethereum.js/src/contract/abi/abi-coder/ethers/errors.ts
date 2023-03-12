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
