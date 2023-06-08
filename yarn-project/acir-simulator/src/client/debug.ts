import { ACVMField } from '../acvm/index.js';

/**
 * Convert an array of ACVMFields to a string.
 *
 * @param msg - array of ACVMFields where each represents a single ascii character
 * @returns string representation of the message
 */
function acvmFieldMessageToString(msg: ACVMField[]): string {
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
 * @param fields - either a single field to be printed, or a string to be formatted.
 * When it is a string to be formatted:
 * The last entry in `fields` is `numArgs` (the number of formatting
 * args). The `formatArgs` occupy the end of the `fields` array,
 * excluding that last entry (`numArgs`). The message string `msg`
 * takes up the remaining entries at the start of the `fields` array.
 *
 * @returns formatted string
 */
export function fieldsToFormattedStr(fields: ACVMField[]): string {
  if (fields.length === 1) {
    return `${fields[0]}`;
  } else {
    const numArgs = Number(fields[fields.length - 1]);
    const msgLen = fields.length - 1 - numArgs;

    const msgFields = fields.slice(0, msgLen);
    const formatArgs = fields.slice(msgLen, fields.length - 1);

    const msg = acvmFieldMessageToString(msgFields);
    const formattedMsg = applyStringFormatting(msg, formatArgs);

    return formattedMsg;
  }
}
