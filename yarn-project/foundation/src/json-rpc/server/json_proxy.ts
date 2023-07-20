import { createDebugLogger } from '../../log/index.js';
import { ClassConverter, JsonClassConverterInput, StringClassConverterInput } from '../class_converter.js';
import { convertFromJsonObj, convertToJsonObj } from '../convert.js';
import { assert, hasOwnProperty } from '../js_utils.js';

const debug = createDebugLogger('json-rpc:json_proxy');

/**
 * Handles conversion of objects over the write.
 * Delegates to a ClassConverter object.
 */
export class JsonProxy {
  classConverter: ClassConverter;
  constructor(
    private handler: object,
    stringClassMap: StringClassConverterInput,
    objectClassMap: JsonClassConverterInput,
  ) {
    this.classConverter = new ClassConverter(stringClassMap, objectClassMap);
  }
  /**
   * Call an RPC method.
   * @param methodName - The RPC method.
   * @param jsonParams - The RPG parameters.
   * @returns The remote result.
   */
  public async call(methodName: string, jsonParams: any[] = []) {
    debug(`JsonProxy:call`, methodName, jsonParams);
    // Get access to our class members
    const proto = Object.getPrototypeOf(this.handler);
    assert(hasOwnProperty(proto, methodName), `JsonProxy: Method ${methodName} not found!`);
    assert(Array.isArray(jsonParams), 'JsonProxy: Params not an array!');
    // convert the params from json representation to classes
    const convertedParams = jsonParams.map(param => convertFromJsonObj(this.classConverter, param));
    debug('JsonProxy:call', methodName, '<-', convertedParams);
    const rawRet = await (this.handler as any)[methodName](...convertedParams);
    const ret = convertToJsonObj(this.classConverter, rawRet);
    debug('JsonProxy:call', methodName, '->', ret);
    return ret;
  }
}
