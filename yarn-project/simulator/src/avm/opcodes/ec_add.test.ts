import { Fr, Point } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';

import { beforeEach } from '@jest/globals';

import { type AvmContext } from '../avm_context.js';
import { Field, Uint32 } from '../avm_memory_types.js';
import { initContext } from '../fixtures/index.js';
import { EcAdd } from './ec_add.js';

describe('EC Instructions', () => {
  let context: AvmContext;
  const grumpkin: Grumpkin = new Grumpkin();

  beforeEach(() => {
    context = initContext();
  });

  describe('EcAdd', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        EcAdd.opcode, // opcode
        0x20, // indirect
        ...Buffer.from('12345670', 'hex'), // p1x
        ...Buffer.from('12345671', 'hex'), // p1y
        ...Buffer.from('00000000', 'hex'), // p1IsInfinite
        ...Buffer.from('12345672', 'hex'), // p2x
        ...Buffer.from('12345673', 'hex'), // p2y
        ...Buffer.from('00000001', 'hex'), // p2IsInfinite
        ...Buffer.from('12345674', 'hex'), // dstOffset
      ]);
      const inst = new EcAdd(
        /*indirect=*/ 0x20,
        /*p1X=*/ 0x12345670,
        /*p1Y=*/ 0x12345671,
        /*p1IsInfinite=*/ 0,
        /*p2X=*/ 0x12345672,
        /*p2Y=*/ 0x12345673,
        /*p2IsInfinite=*/ 1,
        /*dstOffset=*/ 0x12345674,
      );

      expect(EcAdd.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it(`Should double correctly`, async () => {
      const x = new Field(grumpkin.generator().x);
      const y = new Field(grumpkin.generator().y);
      const zero = new Uint32(0);

      context.machineState.memory.set(0, x);
      context.machineState.memory.set(1, y);
      context.machineState.memory.set(2, zero);
      context.machineState.memory.set(3, x);
      context.machineState.memory.set(4, y);
      context.machineState.memory.set(5, zero);
      // context.machineState.memory.set(6, new Uint32(6));

      await new EcAdd(
        /*indirect=*/ 0,
        /*p1X=*/ 0,
        /*p1Y=*/ 1,
        /*p1IsInfinite=*/ 2,
        /*p2X=*/ 3,
        /*p2Y=*/ 4,
        /*p2IsInfinite=*/ 5,
        /*dstOffset=*/ 6,
      ).execute(context);

      const pIsInfinite = context.machineState.memory.get(8).toNumber() === 1;
      const actual = new Point(
        context.machineState.memory.get(6).toFr(),
        context.machineState.memory.get(7).toFr(),
        pIsInfinite,
      );
      const expected = grumpkin.add(grumpkin.generator(), grumpkin.generator());
      expect(actual).toEqual(expected);
      expect(context.machineState.memory.get(8).toFr().equals(Fr.ZERO)).toBe(true);
    });

    it('Should add correctly', async () => {
      const G2 = grumpkin.add(grumpkin.generator(), grumpkin.generator());
      const zero = new Uint32(0);

      const x1 = new Field(grumpkin.generator().x);
      const y1 = new Field(grumpkin.generator().y);
      const x2 = new Field(G2.x);
      const y2 = new Field(G2.y);

      context.machineState.memory.set(0, x1);
      context.machineState.memory.set(1, y1);
      context.machineState.memory.set(2, zero);
      context.machineState.memory.set(3, x2);
      context.machineState.memory.set(4, y2);
      context.machineState.memory.set(5, zero);
      context.machineState.memory.set(6, new Uint32(6));

      await new EcAdd(
        /*indirect=*/ 0,
        /*p1X=*/ 0,
        /*p1Y=*/ 1,
        /*p1IsInfinite=*/ 2,
        /*p2X=*/ 3,
        /*p2Y=*/ 4,
        /*p2IsInfinite=*/ 5,
        /*dstOffset=*/ 6,
      ).execute(context);

      const actual = new Point(
        context.machineState.memory.get(6).toFr(),
        context.machineState.memory.get(7).toFr(),
        false,
      );
      const G3 = grumpkin.add(grumpkin.generator(), G2);
      expect(actual).toEqual(G3);
      expect(context.machineState.memory.get(8).toFr().equals(Fr.ZERO)).toBe(true);
    });
  });
});
