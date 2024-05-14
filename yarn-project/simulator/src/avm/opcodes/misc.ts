import { applyStringFormatting, createDebugLogger } from '@aztec/foundation/log';

import { type AvmContext } from '../avm_context.js';
import { TypeTag } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class DebugLog extends Instruction {
  static type: string = 'DEBUGLOG';
  static readonly opcode: Opcode = Opcode.DEBUGLOG;
  static readonly logger = createDebugLogger('aztec:avm_simulator:debug_log');

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8, // Opcode
    OperandType.UINT8, // Indirect
    OperandType.UINT32, // message memory address
    OperandType.UINT32, // message size
    OperandType.UINT32, // fields memory address
    OperandType.UINT32, // fields size address
  ];

  constructor(
    private indirect: number,
    private messageOffset: number,
    private messageSize: number,
    private fieldsOffset: number,
    private fieldsSizeOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memory = context.machineState.memory.track(this.type);
    const [messageOffset, fieldsOffset, fieldsSizeOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.messageOffset, this.fieldsOffset, this.fieldsSizeOffset],
      memory,
    );

    const fieldsSize = memory.get(fieldsSizeOffset).toNumber();
    memory.checkTagsRange(TypeTag.UINT8, messageOffset, this.messageSize);
    memory.checkTagsRange(TypeTag.FIELD, fieldsOffset, fieldsSize);

    const memoryOperations = { reads: 1 + fieldsSize + this.messageSize, writes: 0, indirect: this.indirect };
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const rawMessage = memory.getSlice(messageOffset, this.messageSize);
    const fields = memory.getSlice(fieldsOffset, fieldsSize);

    // Interpret str<N> = [u8; N] to string.
    const messageAsStr = rawMessage.map(field => String.fromCharCode(field.toNumber())).join('');
    const formattedStr = applyStringFormatting(
      messageAsStr,
      fields.map(field => field.toFr()),
    );

    DebugLog.logger.verbose(formattedStr);

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
