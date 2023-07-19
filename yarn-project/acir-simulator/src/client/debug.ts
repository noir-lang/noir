import { ForeignCallInput } from 'acvm_js';

import { ACVMField } from '../acvm/index.js';

/**
 * Convert an array of ACVMFields to a string.
 *
 * @param msg - array of ACVMFields where each represents a single ascii character
 * @returns string representation of the message
 */
export function acvmFieldMessageToString(msg: ACVMField[]): string {
  let msgStr = '';
  for (const msgChar of msg) {
    const asciiCode = Number(msgChar);
    const asciiChar = String.fromCharCode(asciiCode);
    msgStr = msgStr.concat(asciiChar);
  }
  // cut off string in case of preemptive null termination
  const nullCharIndex = msgStr.indexOf('\\0');
  if (nullCharIndex >= 0) {
    msgStr = msgStr.substring(0, nullCharIndex);
  }
  return msgStr.replaceAll('\\n', '\n').replaceAll('\\t', '\t');
}

/**
 * Format a debug string for Noir filling in `'{0}'` entries with their
 * corresponding values from the args array.
 *
 * @param formatStr - str of form `'this is a string with some entries like {0} and {1}'`
 * @param args - array of fields to fill in the string format entries with
 * @returns formatted string
 */
function applyStringFormatting(formatStr: string, args: ACVMField[]): string {
  const matches = formatStr.match(/{\d+}/g);
  if (matches == null) {
    return formatStr;
  }
  // Get the numeric values within the curly braces, convert them to numbers,
  // and find the maximum value.
  const maxIndex = Math.max(...matches.map(match => Number(match.slice(1, -1))));
  const argsPadded = args.concat(Array.from({ length: Math.max(0, maxIndex - args.length) }, () => '0xBAD'));

  return formatStr.replace(/{(\d+)}/g, function (match, index) {
    return typeof args[index] != 'undefined' ? argsPadded[index] : match;
  });
}

/**
 * Convert an array of ACVMFields from ACVM to a formatted string.
 *
 * @param parameters - either one parameter representing a simple field or array, or two parameters when
 * It's a message without args or three parameters when it's a message with arguments.
 *
 * @returns formatted string
 */
export function oracleDebugCallToFormattedStr(parameters: ForeignCallInput[]): string {
  if (parameters.length === 1) {
    return processFieldOrArray(parameters[0]);
  }

  let formatArgs: string[] = [];

  if (parameters.length > 2) {
    formatArgs = parameters[1];
  }

  const formattedMsg = applyStringFormatting(acvmFieldMessageToString(parameters[0]), formatArgs);

  return formattedMsg;
}

/**
 * Processes a field or an array and returns a string representation.
 * @param fieldOrArray - The field or array to be processed.
 * @returns Returns the processed string representation of the field or array.
 */
function processFieldOrArray(fieldOrArray: string[]) {
  if (fieldOrArray.length === 1) {
    return `${fieldOrArray[0]}`;
  }

  // Check if all the elements start with 63 zero bytes
  // --> if yes, we have an array of bytes and we print as hex
  if (onlyBytes(fieldOrArray)) {
    const decimalArray = fieldOrArray.map(element => parseInt(element, 16));
    return '0x' + Buffer.from(decimalArray).toString('hex');
  }

  return '[' + fieldOrArray.join(', ') + ']';
}

/**
 * Checks if all elements in the array are valid byte representations.
 * @param array - The array to be checked.
 * @returns Returns `true` if all elements are valid byte representations, `false` otherwise.
 * @throws Throws an error if any element has an invalid length.
 */
function onlyBytes(array: string[]): boolean {
  for (const element of array) {
    if (element.length != 66) {
      throw new Error('Invalid element length. Expected 66 chars, got ' + element.length + ' chars.');
    }
    if (!element.startsWith('0x00000000000000000000000000000000000000000000000000000000000000')) {
      return false;
    }
  }
  return true;
}
