import { keccak256String } from '../../../crypto/index.js';
import { AbiCoder as EthersAbi } from './ethers/abi-coder.js';
import { AbiInput } from '../contract_abi_definition.js';
import { hexToBuffer } from '../../../hex_string/index.js';

/**
 * ABICoder prototype should be used to encode/decode solidity params of any type.
 */
export class ABICoder {
  private ethersAbiCoder: EthersAbi;

  constructor() {
    this.ethersAbiCoder = new EthersAbi();
  }

  /**
   * EncodeFunctionSignature.
   * Encodes the function name to its ABI representation, which are the first 4 bytes of the sha3 of the function name including  types.
   * @param functionName - Name of the function.
   * @returns Encoded function name.
   */
  public encodeFunctionSignature(functionName) {
    if (typeof functionName === 'object') {
      functionName = this.abiMethodToString(functionName);
    }

    return '0x' + keccak256String(functionName).slice(0, 8);
  }

  /**
   * Encodes the function name to its ABI representation, which are the first 4 bytes of the sha3 of the function name including  types.
   *EncodeEventSignature.
   * @param functionName - Name of the function.
   * @returns Encoded function name.
   */
  public encodeEventSignature(functionName) {
    if (typeof functionName === 'object') {
      functionName = this.abiMethodToString(functionName);
    }

    return '0x' + keccak256String(functionName);
  }

  /**
   * Should be used to encode plain param.
   * EncodeParameter.
   * @param type - Type of the param.
   * @param param - Param to be encoded.
   * @returns Encoded plain param.
   */
  public encodeParameter(type, param) {
    return this.encodeParameters([type], [param]);
  }

  /**
   * Should be used to encode list of params.
   * EncodeParameters.
   * @param types - Array of types.
   * @param params - Params to be encoded.
   * @returns Encoded list of params.
   */
  public encodeParameters(types, params) {
    return hexToBuffer(this.ethersAbiCoder.encode(this.mapTypes(types), params));
  }

  /**
   * Encodes a function call from its json interface and parameters.
   * EncodeFunctionCall.
   * @param jsonInterface - Interface of the function call represented in JSON.
   * @param params - Parameters of the function call.
   * @returns The encoded ABI for this function call.
   */
  public encodeFunctionCall(jsonInterface, params) {
    return (
      this.encodeFunctionSignature(jsonInterface) + this.encodeParameters(jsonInterface.inputs, params).toString('hex')
    );
  }

  /**
   * Should be used to decode bytes to plain param.
   * DecodeParameter.
   * @param type - Of param.
   * @param bytes - To be decoded.
   * @returns Plain param.
   */
  public decodeParameter(type, bytes: Buffer | string) {
    return this.decodeParameters([type], bytes)[0];
  }

  /**
   * Should be used to decode list of params.
   * @returns Decoded list of params.
   */
  public decodeParameters(outputs, bytes: Buffer | string): { [k: string | number]: any } {
    const returnValue: { [k: string | number]: any } = { __length__: 0 };

    if (typeof bytes === 'string') {
      bytes = hexToBuffer(bytes);
    }

    if (!bytes || bytes.length === 0) {
      if (outputs.length === 0) {
        return returnValue;
      }
      throw new Error('decodeParameters: Buffer length is 0.');
    }

    const res = this.ethersAbiCoder.decode(this.mapTypes(outputs), bytes);

    outputs.forEach((output, i) => {
      let decodedValue = res[returnValue.__length__];
      decodedValue = decodedValue === '0x' ? null : decodedValue;

      returnValue[i] = decodedValue;

      if (typeof output === 'object' && output.name) {
        returnValue[output.name] = decodedValue;
      }

      returnValue.__length__++;
    });

    return returnValue;
  }

  /**
   * Decodes events non- and indexed parameters.
   * DecodeLog.
   * @param inputs - Abi inputs.
   * @param data - Event data.
   * @param topics - Event topics.
   * @returns Array of plain params.
   */
  public decodeLog(inputs: AbiInput[], data, topics) {
    topics = Array.isArray(topics) ? topics : [topics];

    data = data || '';

    const notIndexedInputs: any[] = [];
    const indexedParams: any[] = [];
    let topicCount = 0;

    // TODO check for anonymous logs?

    inputs.forEach((input, i) => {
      if (input.indexed) {
        indexedParams[i] = ['bool', 'int', 'uint', 'address', 'fixed', 'ufixed'].some(t => input.type.includes(t))
          ? this.decodeParameter(input.type, topics[topicCount])
          : topics[topicCount];
        topicCount++;
      } else {
        notIndexedInputs[i] = input;
      }
    });

    const nonIndexedData = data;
    const notIndexedParams =
      nonIndexedData && nonIndexedData !== '0x' ? this.decodeParameters(notIndexedInputs, nonIndexedData) : [];

    const returnValue: any = {};
    returnValue.__length__ = 0;

    inputs.forEach((res, i) => {
      returnValue[i] = res.type === 'string' ? '' : null;

      if (typeof notIndexedParams[i] !== 'undefined') {
        returnValue[i] = notIndexedParams[i];
      }
      if (typeof indexedParams[i] !== 'undefined') {
        returnValue[i] = indexedParams[i];
      }

      if (res.name) {
        returnValue[res.name] = returnValue[i];
      }

      returnValue.__length__++;
    });

    return returnValue;
  }

  /**
   * Map types if simplified format is used.
   * MapTypes.
   * @param types - Types to be mapped.
   * @returns Array.
   */
  private mapTypes(types) {
    const mappedTypes: any[] = [];
    types.forEach(type => {
      if (this.isSimplifiedStructFormat(type)) {
        const structName = Object.keys(type)[0];
        mappedTypes.push(
          Object.assign(this.mapStructNameAndType(structName), {
            components: this.mapStructToCoderFormat(type[structName]),
          }),
        );

        return;
      }

      mappedTypes.push(type);
    });

    return mappedTypes;
  }

  /**
   * Check if type is simplified struct format.
   * IsSimplifiedStructFormat.
   * @param type - The type.
   * @returns A boolean.
   */
  private isSimplifiedStructFormat(type) {
    return typeof type === 'object' && typeof type.components === 'undefined' && typeof type.name === 'undefined';
  }

  /**
   * Maps the correct tuple type and name when the simplified format in encode/decodeParameter is used.
   * MapStructNameAndType.
   * @param structName -  Name.
   * @returns \{type: string, name: *\}.
   */
  private mapStructNameAndType(structName) {
    let type = 'tuple';

    if (structName.indexOf('[]') > -1) {
      type = 'tuple[]';
      structName = structName.slice(0, -2);
    }

    return { type, name: structName };
  }

  /**
   * Maps the simplified format in to the expected format of the ABICoder.
   * MapStructToCoderFormat.
   * @param struct - Simplified format.
   * @returns Array.
   */
  private mapStructToCoderFormat(struct) {
    const components: any[] = [];
    Object.keys(struct).forEach(key => {
      if (typeof struct[key] === 'object') {
        components.push(
          Object.assign(this.mapStructNameAndType(key), {
            components: this.mapStructToCoderFormat(struct[key]),
          }),
        );

        return;
      }

      components.push({
        name: key,
        type: struct[key],
      });
    });

    return components;
  }

  /**
   * Should be used to create full function/event name from json abi.
   * JsonInterfaceMethodToString.
   * @param json - JSON abi.
   * @returns Full function/event name.
   */
  public abiMethodToString(json) {
    if (typeof json === 'object' && json.name && json.name.indexOf('(') !== -1) {
      return json.name;
    }

    return json.name + '(' + flattenTypes(false, json.inputs).join(',') + ')';
  }
}

/**
 * Should be used to flatten json abi inputs/outputs into an array of type-representing-strings.
 * FlattenTypes.
 * @param includeTuple - Bool.
 * @param puts - Inputs/outputs.
 * @returns Array of Parameters as strings.
 */
function flattenTypes(includeTuple: boolean, puts: any[]) {
  // console.log("entered _flattenTypes. inputs/outputs: " + puts)
  const types: any[] = [];

  puts.forEach(param => {
    if (typeof param.components === 'object') {
      if (param.type.substring(0, 5) !== 'tuple') {
        throw new Error('components found but type is not tuple; report on GitHub');
      }
      let suffix = '';
      const arrayBracket = param.type.indexOf('[');
      if (arrayBracket >= 0) {
        suffix = param.type.substring(arrayBracket);
      }
      const result = flattenTypes(includeTuple, param.components);
      // console.log("result should have things: " + result)
      if (Array.isArray(result) && includeTuple) {
        // console.log("include tuple word, and its an array. joining...: " + result.types)
        types.push('tuple(' + result.join(',') + ')' + suffix);
      } else if (!includeTuple) {
        // console.log("don't include tuple, but its an array. joining...: " + result)
        types.push('(' + result.join(',') + ')' + suffix);
      } else {
        // console.log("its a single type within a tuple: " + result.types)
        types.push('(' + result + ')');
      }
    } else {
      // console.log("its a type and not directly in a tuple: " + param.type)
      types.push(param.type);
    }
  });

  return types;
}

export const abiCoder = new ABICoder();
