import { AztecAddress, type Fr } from '@aztec/circuits.js';
import { type AbiType, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { type FieldsOf } from '@aztec/foundation/types';

/** A request to call a function on a contract from a given address. */
export class FunctionCall {
  constructor(
    /** The name of the function to call */
    public name: string,
    /** The recipient contract */
    public to: AztecAddress,
    /** The function being called */
    public selector: FunctionSelector,
    /** Type of the function */
    public type: FunctionType,
    /** Whether this call can make modifications to state or not */
    public isStatic: boolean,
    /** The encoded args */
    public args: Fr[],
    /** The return type for decoding */
    public returnTypes: AbiType[],
  ) {}

  static getFields(fields: FieldsOf<FunctionCall>) {
    return [
      fields.name,
      fields.to,
      fields.selector,
      fields.type,
      fields.isStatic,
      fields.args,
      fields.returnTypes,
    ] as const;
  }

  static from(fields: FieldsOf<FunctionCall>): FunctionCall {
    return new FunctionCall(...FunctionCall.getFields(fields));
  }

  /**
   * Creates an empty function call.
   * @returns an empty function call.
   */
  public static empty() {
    return {
      name: '',
      to: AztecAddress.ZERO,
      selector: FunctionSelector.empty(),
      type: FunctionType.PUBLIC,
      isStatic: false,
      args: [],
      returnTypes: [],
    };
  }
}
