import { Fr } from '@aztec/foundation/fields';

export type ForeignCallSingle = string;

export type ForeignCallArray = string[];

export type ForeignCallResult = {
  values: (ForeignCallSingle | ForeignCallArray)[];
};

export function fromSingle(obj: ForeignCallSingle) {
  return Fr.fromBuffer(Buffer.from(obj, 'hex'));
}

export function fromArray(obj: ForeignCallArray) {
  return obj.map(str => Fr.fromBuffer(Buffer.from(str, 'hex')));
}

export function toSingle(obj: Fr) {
  return obj.toString().slice(2);
}

export function toArray(objs: Fr[]) {
  return objs.map(obj => obj.toString());
}

export function toForeignCallResult(obj: (ForeignCallSingle | ForeignCallArray)[]) {
  return { values: obj };
}
