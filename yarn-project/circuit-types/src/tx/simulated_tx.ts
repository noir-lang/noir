import { Fr } from '@aztec/circuits.js';

import { Tx } from './tx.js';

export type ProcessReturnValues = Fr[] | undefined;

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
    const returnToJson = (data: ProcessReturnValues | undefined): string => {
      if (data === undefined) {
        return JSON.stringify(data);
      }
      return JSON.stringify(data.map(fr => fr.toString()));
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
    const returnFromJson = (json: string): ProcessReturnValues | undefined => {
      if (json === undefined) {
        return json;
      }
      return JSON.parse(json).map(Fr.fromString);
    };

    const tx = Tx.fromJSON(obj.tx);
    const privateReturnValues = returnFromJson(obj.privateReturnValues);
    const publicReturnValues = returnFromJson(obj.publicReturnValues);

    return new SimulatedTx(tx, privateReturnValues, publicReturnValues);
  }
}
