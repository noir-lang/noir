import { Keccak, Pedersen, Poseidon2, Sha256 } from '../opcodes/hashing.js';
import {
  Add,
  Address,
  And,
  BlockNumber,
  CMov,
  Call,
  CalldataCopy,
  Cast,
  ChainId,
  Div,
  EmitNoteHash,
  EmitNullifier,
  EmitUnencryptedLog,
  Eq,
  FeePerDAGas,
  FeePerL1Gas,
  FeePerL2Gas,
  FieldDiv,
  GetContractInstance,
  InternalCall,
  InternalReturn,
  Jump,
  JumpI,
  L1ToL2MessageExists,
  Lt,
  Lte,
  Mov,
  Mul,
  Not,
  NoteHashExists,
  NullifierExists,
  Or,
  Origin,
  Portal,
  Return,
  Revert,
  SLoad,
  SStore,
  SendL2ToL1Message,
  Sender,
  Set,
  Shl,
  Shr,
  StaticCall,
  StorageAddress,
  Sub,
  Timestamp,
  Version,
  Xor,
} from '../opcodes/index.js';
import type { Instruction } from '../opcodes/index.js';
import { BufferCursor } from './buffer_cursor.js';
import { Opcode } from './instruction_serialization.js';

interface DeserializableInstruction {
  deserialize(buf: BufferCursor | Buffer): Instruction;
  opcode: Opcode;
}

export type InstructionSet = Map<Opcode, DeserializableInstruction>;
// TODO(4359): This is a function so that Call and StaticCall can be lazily resolved.
// This is a temporary solution until we solve the dependency cycle.
const INSTRUCTION_SET = () =>
  new Map<Opcode, DeserializableInstruction>([
    [Add.opcode, Add],
    [Sub.opcode, Sub],
    [Mul.opcode, Mul],
    [Div.opcode, Div],
    [FieldDiv.opcode, FieldDiv],
    [Eq.opcode, Eq],
    [Lt.opcode, Lt],
    [Lte.opcode, Lte],
    [And.opcode, And],
    [Or.opcode, Or],
    [Xor.opcode, Xor],
    [Not.opcode, Not],
    [Shl.opcode, Shl],
    [Shr.opcode, Shr],
    [Cast.opcode, Cast],
    [Address.opcode, Address],
    [StorageAddress.opcode, StorageAddress],
    [Origin.opcode, Origin],
    [Sender.opcode, Sender],
    [Portal.opcode, Portal],
    [FeePerL1Gas.opcode, FeePerL1Gas],
    [FeePerL2Gas.opcode, FeePerL2Gas],
    [FeePerDAGas.opcode, FeePerDAGas],
    //[Contractcalldepth.opcode, Contractcalldepth],
    // Execution Environment - Globals
    [ChainId.opcode, ChainId],
    [Version.opcode, Version],
    [BlockNumber.opcode, BlockNumber],
    [Timestamp.opcode, Timestamp],
    //[Coinbase.opcode, Coinbase],
    //[Blockl1gaslimit.opcode, Blockl1gaslimit],
    //[Blockl2gaslimit.opcode, Blockl2gaslimit],
    //[Blockdagaslimit.opcode, Blockdagaslimit],
    // Execution Environment - Calldata
    [CalldataCopy.opcode, CalldataCopy],

    // Machine State
    // Machine State - Gas
    //[L1gasleft.opcode, L1gasleft],
    //[L2gasleft.opcode, L2gasleft],
    //[Dagasleft.opcode, Dagasleft],
    // Machine State - Internal Control Flow
    [Jump.opcode, Jump],
    [JumpI.opcode, JumpI],
    [InternalCall.opcode, InternalCall],
    [InternalReturn.opcode, InternalReturn],
    [Set.opcode, Set],
    [Mov.opcode, Mov],
    [CMov.opcode, CMov],

    // World State
    [SLoad.opcode, SLoad], // Public Storage
    [SStore.opcode, SStore], // Public Storage
    [NoteHashExists.opcode, NoteHashExists], // Notes & Nullifiers
    [EmitNoteHash.opcode, EmitNoteHash], // Notes & Nullifiers
    [NullifierExists.opcode, NullifierExists], // Notes & Nullifiers
    [EmitNullifier.opcode, EmitNullifier], // Notes & Nullifiers
    [L1ToL2MessageExists.opcode, L1ToL2MessageExists], // Messages
    //[HeaderMember.opcode, HeaderMember], // Header

    // Accrued Substate
    [EmitUnencryptedLog.opcode, EmitUnencryptedLog],
    [SendL2ToL1Message.opcode, SendL2ToL1Message],
    [GetContractInstance.opcode, GetContractInstance],

    // Control Flow - Contract Calls
    [Call.opcode, Call],
    [StaticCall.opcode, StaticCall],
    //[DelegateCall.opcode, DelegateCall],
    [Return.opcode, Return],
    [Revert.opcode, Revert],

    // //// Gadgets
    [Keccak.opcode, Keccak],
    [Poseidon2.opcode, Poseidon2],
    [Sha256.opcode, Sha256],
    [Pedersen.opcode, Pedersen],
  ]);

interface Serializable {
  serialize(): Buffer;
}

/**
 * Serializes an array of instructions to bytecode.
 */
export function encodeToBytecode(instructions: Serializable[]): Buffer {
  return Buffer.concat(instructions.map(i => i.serialize()));
}

/**
 * Convert a buffer of bytecode into an array of instructions.
 * @param bytecode Buffer of bytecode.
 * @param instructionSet Optional {@code InstructionSet} to be used for deserialization.
 * @returns Bytecode decoded into an ordered array of Instructions
 */
export function decodeFromBytecode(
  bytecode: Buffer,
  instructionSet: InstructionSet = INSTRUCTION_SET(),
): Instruction[] {
  const instructions: Instruction[] = [];
  const cursor = new BufferCursor(bytecode);

  while (!cursor.eof()) {
    const opcode: Opcode = cursor.bufferAtPosition().readUint8(); // peek.
    const instructionDeserializerOrUndef = instructionSet.get(opcode);
    if (instructionDeserializerOrUndef === undefined) {
      throw new Error(`Opcode ${Opcode[opcode]} (0x${opcode.toString(16)}) not implemented`);
    }

    const instructionDeserializer: DeserializableInstruction = instructionDeserializerOrUndef;
    const i: Instruction = instructionDeserializer.deserialize(cursor);
    instructions.push(i);
  }

  return instructions;
}
