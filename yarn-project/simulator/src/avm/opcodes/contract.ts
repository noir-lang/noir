import { AztecAddress, Fr } from '@aztec/circuits.js';

import type { AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class GetContractInstance extends Instruction {
  static readonly type: string = 'GETCONTRACTINSTANCE';
  static readonly opcode: Opcode = Opcode.GETCONTRACTINSTANCE;
  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8,
    OperandType.UINT8,
    OperandType.UINT32,
    OperandType.UINT32,
  ];

  constructor(private indirect: number, private addressOffset: number, private dstOffset: number) {
    super();
  }

  async execute(context: AvmContext): Promise<void> {
    const [addressOffset, dstOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.addressOffset, this.dstOffset],
      context.machineState.memory,
    );

    const address = AztecAddress.fromField(context.machineState.memory.get(addressOffset).toFr());
    const instance = await context.persistableState.hostStorage.contractsDb.getContractInstance(address);

    const data =
      instance === undefined
        ? [
            new Field(0), // not found
            new Field(0),
            new Field(0),
            new Field(0),
            new Field(0),
            new Field(0),
            new Field(0),
          ]
        : [
            new Fr(1), // found
            instance.salt,
            instance.deployer.toField(),
            instance.contractClassId,
            instance.initializationHash,
            instance.portalContractAddress.toField(),
            instance.publicKeysHash,
          ].map(f => new Field(f));

    context.machineState.memory.setSlice(dstOffset, data);

    context.machineState.incrementPc();
  }
}
