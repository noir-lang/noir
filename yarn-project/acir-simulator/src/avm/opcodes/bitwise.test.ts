import { Fr } from '@aztec/foundation/fields';

import { mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import {
  And,
  /*Not,*/
  Or,
  Shl,
  Shr,
  Xor,
} from './bitwise.js';

describe('Bitwise instructions', () => {
  let machineState: AvmMachineState;
  let stateManager = mock<AvmStateManager>();

  beforeEach(() => {
    machineState = new AvmMachineState([]);
    stateManager = mock<AvmStateManager>();
  });

  it('Should AND correctly over Fr type', () => {
    const a = new Fr(0b11111110010011100100n);
    const b = new Fr(0b11100100111001001111n);

    machineState.writeMemory(0, a);
    machineState.writeMemory(1, b);

    new And(0, 1, 2).execute(machineState, stateManager);

    const expected = new Fr(0b11100100010001000100n);
    const actual = machineState.readMemory(2);
    expect(actual).toEqual(expected);
  });

  it('Should OR correctly over Fr type', () => {
    const a = new Fr(0b11111110010011100100n);
    const b = new Fr(0b11100100111001001111n);

    machineState.writeMemory(0, a);
    machineState.writeMemory(1, b);

    new Or(0, 1, 2).execute(machineState, stateManager);

    const expected = new Fr(0b11111110111011101111n);
    const actual = machineState.readMemory(2);
    expect(actual).toEqual(expected);
  });

  it('Should XOR correctly over Fr type', () => {
    const a = new Fr(0b11111110010011100100n);
    const b = new Fr(0b11100100111001001111n);

    machineState.writeMemory(0, a);
    machineState.writeMemory(1, b);

    new Xor(0, 1, 2).execute(machineState, stateManager);

    const expected = new Fr(0b00011010101010101011n);
    const actual = machineState.readMemory(2);
    expect(actual).toEqual(expected);
  });

  describe('SHR', () => {
    it('Should shift correctly 0 positions over Fr type', () => {
      const a = new Fr(0b11111110010011100100n);
      const b = new Fr(0n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Shr(0, 1, 2).execute(machineState, stateManager);

      const expected = a;
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });

    it('Should shift correctly 2 positions over Fr type', () => {
      const a = new Fr(0b11111110010011100100n);
      const b = new Fr(2n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Shr(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(0b00111111100100111001n);
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });

    it('Should shift correctly 19 positions over Fr type', () => {
      const a = new Fr(0b11111110010011100100n);
      const b = new Fr(19n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Shr(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(0b01n);
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('SHL', () => {
    it('Should shift correctly 0 positions over Fr type', () => {
      const a = new Fr(0b11111110010011100100n);
      const b = new Fr(0n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Shl(0, 1, 2).execute(machineState, stateManager);

      const expected = a;
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });

    it('Should shift correctly 2 positions over Fr type', () => {
      const a = new Fr(0b11111110010011100100n);
      const b = new Fr(2n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Shl(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(0b1111111001001110010000n);
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });

    // it('Should shift correctly over bit limit over Fr type', () => {
    //   const a = new Fr(0b11111110010011100100n);
    //   const b = new Fr(19n);

    //   machineState.writeMemory(0, a);
    //   machineState.writeMemory(1, b);

    //   new Shl(0, 1, 2).execute(machineState, stateManager);

    //   const expected = new Fr(0b01n);
    //   const actual = machineState.readMemory(2);
    //   expect(actual).toEqual(expected);
    // });
  });

  // it('Should NOT correctly over Fr type', () => {
  //   const a = new Fr(0b11111110010011100100n);

  //   machineState.writeMemory(0, a);

  //   new Not(0, 1).execute(machineState, stateManager);

  //   const expected = new Fr(0b00000001101100011011n); // high bits!
  //   const actual = machineState.readMemory(1);
  //   expect(actual).toEqual(expected);
  // });
});
