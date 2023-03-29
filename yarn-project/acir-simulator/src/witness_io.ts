import { ACVMWitness, fromACVMField, toACVMField } from './acvm.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';

export class WitnessReader {
  constructor(private currentIndex: number, private witness: ACVMWitness) {}

  public readField(): Fr {
    const field = this.witness.get(this.currentIndex);
    if (!field) {
      throw new Error(`Missing field at index ${this.currentIndex}`);
    }

    this.currentIndex += 1;
    return fromACVMField(field);
  }

  public readFieldArray(length: number): Fr[] {
    const array: Fr[] = [];
    for (let i = 0; i < length; i++) {
      array.push(this.readField());
    }
    return array;
  }
}

export class WitnessWriter {
  constructor(private currentIndex: number, private witness: ACVMWitness) {}

  public writeField(field: Parameters<typeof toACVMField>[0]) {
    this.witness.set(this.currentIndex, toACVMField(field));
    this.currentIndex += 1;
  }

  public writeFieldArray(array: Fr[]) {
    for (const field of array) {
      this.writeField(field);
    }
  }

  public jump(amount: number) {
    this.currentIndex += amount;
  }
}

export function frToAztecAddress(fr: Fr): AztecAddress {
  return new AztecAddress(fr.toBuffer());
}

export function frToEthAddress(fr: Fr): EthAddress {
  return new EthAddress(fr.toBuffer().slice(-EthAddress.SIZE_IN_BYTES));
}

export function frToBoolean(fr: Fr): boolean {
  const buf = fr.toBuffer();
  return buf[buf.length - 1] !== 0;
}
