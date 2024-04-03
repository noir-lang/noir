import { AztecAddress } from '@aztec/circuits.js';
import { type ProcessReturnValues } from '@aztec/foundation/abi';

import { Tx } from './tx.js';

export class SimulatedTx {
  constructor(
    public tx: Tx,
    public privateReturnValues?: ProcessReturnValues,
    public publicReturnValues?: ProcessReturnValues,
  ) {}

  /**
   * Convert a SimulatedTx class object to a plain JSON object.
   * @returns A plain object with SimulatedTx properties.
   */
  public toJSON() {
    const returnToJson = (data: ProcessReturnValues): string => {
      const replacer = (key: string, value: any): any => {
        if (typeof value === 'bigint') {
          return value.toString() + 'n'; // Indicate bigint with "n"
        } else if (value instanceof AztecAddress) {
          return value.toString();
        } else {
          return value;
        }
      };
      return JSON.stringify(data, replacer);
    };

    return {
      tx: this.tx.toJSON(),
      privateReturnValues: returnToJson(this.privateReturnValues),
      publicReturnValues: returnToJson(this.publicReturnValues),
    };
  }

  /**
   * Convert a plain JSON object to a Tx class object.
   * @param obj - A plain Tx JSON object.
   * @returns A Tx class object.
   */
  public static fromJSON(obj: any) {
    const returnFromJson = (json: string): ProcessReturnValues => {
      if (json == undefined) {
        return json;
      }
      const reviver = (key: string, value: any): any => {
        if (typeof value === 'string') {
          if (value.match(/\d+n$/)) {
            // Detect bigint serialization
            return BigInt(value.slice(0, -1));
          } else if (value.match(/^0x[a-fA-F0-9]{64}$/)) {
            return AztecAddress.fromString(value);
          }
        }
        return value;
      };
      return JSON.parse(json, reviver);
    };

    const tx = Tx.fromJSON(obj.tx);
    const privateReturnValues = returnFromJson(obj.privateReturnValues);
    const publicReturnValues = returnFromJson(obj.publicReturnValues);

    return new SimulatedTx(tx, privateReturnValues, publicReturnValues);
  }
}
