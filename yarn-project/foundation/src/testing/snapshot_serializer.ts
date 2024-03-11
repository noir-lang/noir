import { type NewPlugin } from 'pretty-format';
import { inspect } from 'util';

function makeSerializerForCustomInspect(prefix: string): NewPlugin {
  return {
    serialize(val: any): string {
      return inspect(val);
    },

    test(arg: any): boolean {
      return inspect(arg).startsWith(prefix);
    },
  };
}

const bufferSerializer: NewPlugin = {
  serialize(val: any): string {
    return `Buffer<0x${val.toString('hex')}>`;
  },

  test(arg: any): boolean {
    return Buffer.isBuffer(arg);
  },
};

const CUSTOM_INSPECT_SERIALIZABLE_TYPES = ['AztecAddress', 'Fr', 'Fq', 'Selector', 'EthAddress'];

export function setupCustomSnapshotSerializers(expect: { addSnapshotSerializer: (serializer: NewPlugin) => void }) {
  for (const type of CUSTOM_INSPECT_SERIALIZABLE_TYPES) {
    expect.addSnapshotSerializer(makeSerializerForCustomInspect(type));
  }
  expect.addSnapshotSerializer(bufferSerializer);
}
