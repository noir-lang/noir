// Check if all of the input values are correct according to the ABI
export function validateInputs(inputs, abi) {
  for (const param of abi.parameters) {
    const inputValue = inputs[param.name];
    if (inputValue === undefined) {
      // This is checked by noirc_abi, so we could likely remove this check
      return { isValid: false, error: `Input for ${param.name} is missing` };
    }
    if (!checkType(inputValue, param.type)) {
      return {
        isValid: false,
        error: `Input for ${param.name} is the wrong type, expected ${type_to_string(param.type)}, got "${inputValue}"`,
      };
    }
  }
  return { isValid: true, error: null };
}

// Checks that value is of type "type"
// Where type is taken from the abi
function checkType(value, type) {
  switch (type.kind) {
    case 'integer':
      if (type.sign === 'unsigned') {
        return isUnsignedInteger(value, type.width);
      }
      // Other integer sign checks can be added here
      break;
    // Other type.kind checks can be added here
  }
  return false;
}

function type_to_string(type): string {
  switch (type.kind) {
    case 'integer':
      if (type.sign === 'unsigned') {
        return `uint${type.width}`;
      }
      break;
    case 'array':
      return `${type_to_string(type.element)}[${type.length}]`;
  }
  return 'unknown type';
}

// Returns true if `value` is an unsigned integer that is less than 2^{width}
function isUnsignedInteger(value: bigint, width: bigint) {
  try {
    const bigIntValue = BigInt(value);
    return bigIntValue >= 0 && bigIntValue <= BigInt(2) ** BigInt(width) - 1n;
  } catch (e) {
    return false; // Not a valid integer
  }
}
