import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader } from '@aztec/foundation/serialize';

import { inspect } from 'util';

enum RevertCodeEnum {
  OK = 0,
  APP_LOGIC_REVERTED = 1,
  TEARDOWN_REVERTED = 2,
  BOTH_REVERTED = 3,
}

function isRevertCodeEnum(value: number): value is RevertCodeEnum {
  return (
    value === RevertCodeEnum.OK ||
    value === RevertCodeEnum.APP_LOGIC_REVERTED ||
    value === RevertCodeEnum.TEARDOWN_REVERTED ||
    value === RevertCodeEnum.BOTH_REVERTED
  );
}

/**
 * Wrapper class over a field to safely represent a revert code.
 */
export class RevertCode {
  private code: number;
  private constructor(e: RevertCodeEnum) {
    this.code = e.valueOf();
  }
  static readonly OK: RevertCode = new RevertCode(RevertCodeEnum.OK);
  static readonly APP_LOGIC_REVERTED: RevertCode = new RevertCode(RevertCodeEnum.APP_LOGIC_REVERTED);
  static readonly TEARDOWN_REVERTED: RevertCode = new RevertCode(RevertCodeEnum.TEARDOWN_REVERTED);
  static readonly BOTH_REVERTED: RevertCode = new RevertCode(RevertCodeEnum.BOTH_REVERTED);

  public equals(other: RevertCode): boolean {
    return this.code === other.code;
  }

  public isOK(): boolean {
    return this.equals(RevertCode.OK);
  }

  public getDescription() {
    switch (this.code) {
      case RevertCodeEnum.OK:
        return 'OK';
      case RevertCodeEnum.APP_LOGIC_REVERTED:
        return 'Application logic reverted';
      case RevertCodeEnum.TEARDOWN_REVERTED:
        return 'Teardown reverted';
      case RevertCodeEnum.BOTH_REVERTED:
        return 'Both reverted';
      default:
        return `Unknown RevertCode: ${this.code}`;
    }
  }

  /**
   * Having different serialization methods allows for
   * decoupling the serialization for producing the content commitment hash
   * (where we use fields)
   * from serialization for transmitting the data.
   */

  private static readonly PREIMAGE_SIZE_IN_BYTES = 32;
  public toHashPreimage(): Buffer {
    const padding = Buffer.alloc(RevertCode.PREIMAGE_SIZE_IN_BYTES - RevertCode.PACKED_SIZE_IN_BYTES);
    return Buffer.concat([padding, this.toBuffer()]);
  }

  private static readonly PACKED_SIZE_IN_BYTES = 1;
  public toBuffer(): Buffer {
    const b = Buffer.alloc(RevertCode.PACKED_SIZE_IN_BYTES);
    b.writeUInt8(this.code, 0);
    return b;
  }

  public toField(): Fr {
    return new Fr(this.toBuffer());
  }

  public getSerializedLength(): number {
    return this.toBuffer().length;
  }

  public static fromField(fr: Fr): RevertCode {
    if (!isRevertCodeEnum(fr.toNumber())) {
      throw new Error(`Invalid RevertCode: ${fr.toNumber()}`);
    }
    return new RevertCode(fr.toNumber());
  }

  public static fromFields(fields: Fr[] | FieldReader): RevertCode {
    const reader = FieldReader.asReader(fields);
    return RevertCode.fromField(reader.readField());
  }

  public static fromBuffer(buffer: Buffer | BufferReader): RevertCode {
    const reader = BufferReader.asReader(buffer);
    const code = reader.readBytes(RevertCode.PACKED_SIZE_IN_BYTES).readUInt8(0);
    if (!isRevertCodeEnum(code)) {
      throw new Error(`Invalid RevertCode: ${code}`);
    }
    return new RevertCode(code);
  }

  private static readonly NUM_OPTIONS = 4;
  static random(): RevertCode {
    return new RevertCode(Math.floor(Math.random() * RevertCode.NUM_OPTIONS));
  }

  [inspect.custom]() {
    return `RevertCode<${this.code.toString()}>`;
  }
}
