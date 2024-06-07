import { Fr } from '@aztec/circuits.js';

import type { AvmContext } from '../avm_context.js';
import { Field, TypeTag } from '../avm_memory_types.js';
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
    const memoryOperations = { reads: 1, writes: 6, indirect: this.indirect };
    const memory = context.machineState.memory.track(this.type);
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const [addressOffset, dstOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.addressOffset, this.dstOffset],
      memory,
    );
    memory.checkTag(TypeTag.FIELD, addressOffset);

    const address = memory.get(addressOffset).toFr();
    const instance = await context.persistableState.getContractInstance(address);

    const data = [
      new Fr(instance.exists),
      instance.salt,
      instance.deployer.toField(),
      instance.contractClassId,
      instance.initializationHash,
      instance.publicKeysHash,
    ].map(f => new Field(f));

    memory.setSlice(dstOffset, data);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
