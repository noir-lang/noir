import { pedersenCommit } from '@aztec/foundation/crypto';

import { type AvmContext } from '../avm_context.js';
import { Field, TypeTag, Uint8 } from '../avm_memory_types.js';
import { Opcode, OperandType } from '../serialization/instruction_serialization.js';
import { Addressing } from './addressing_mode.js';
import { Instruction } from './instruction.js';

export class PedersenCommitment extends Instruction {
  static type: string = 'PEDERSENCOMMITMENT';
  static readonly opcode: Opcode = Opcode.PEDERSENCOMMITMENT;

  // Informs (de)serialization. See Instruction.deserialize.
  static readonly wireFormat: OperandType[] = [
    OperandType.UINT8 /* Opcode */,
    OperandType.UINT8 /* Indirect */,
    OperandType.UINT32 /* Input Offset*/,
    OperandType.UINT32 /* Dst Offset */,
    OperandType.UINT32 /* Input Size Offset */,
    OperandType.UINT32 /* Generator Index Offset */,
  ];

  constructor(
    private indirect: number,
    private inputOffset: number,
    private outputOffset: number,
    private inputSizeOffset: number,
    private genIndexOffset: number,
  ) {
    super();
  }

  public async execute(context: AvmContext): Promise<void> {
    const memory = context.machineState.memory.track(this.type);
    const [inputOffset, outputOffset, inputSizeOffset, genIndexOffset] = Addressing.fromWire(this.indirect).resolve(
      [this.inputOffset, this.outputOffset, this.inputSizeOffset, this.genIndexOffset],
      memory,
    );

    const inputSize = memory.get(inputSizeOffset).toNumber();
    memory.checkTag(TypeTag.UINT32, inputSizeOffset);

    const inputs = memory.getSlice(inputOffset, inputSize);
    memory.checkTagsRange(TypeTag.FIELD, inputOffset, inputSize);

    const generatorIndex = memory.get(genIndexOffset).toNumber();
    memory.checkTag(TypeTag.UINT32, genIndexOffset);

    const memoryOperations = { reads: inputSize + 2, writes: 3, indirect: this.indirect };
    context.machineState.consumeGas(this.gasCost(memoryOperations));

    const inputBuffer: Buffer[] = inputs.map(input => input.toBuffer());
    // TODO: Add the generate index to the pedersenCommit function
    const commitment = pedersenCommit(inputBuffer, generatorIndex).map(f => new Field(f));
    // The function doesnt include a flag if the output point is infinity, come back to this
    // for now we just check if theyre zero - until we know how bb encodes them
    const isInfinity = commitment[0].equals(new Field(0)) && commitment[1].equals(new Field(0));

    memory.set(outputOffset, commitment[0]); // Field typed
    memory.set(outputOffset + 1, commitment[1]); // Field typed
    memory.set(outputOffset + 2, new Uint8(isInfinity ? 1 : 0)); // U8 typed

    memory.assert(memoryOperations);
    context.machineState.incrementPc();
  }
}
