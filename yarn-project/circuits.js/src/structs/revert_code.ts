import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader } from '@aztec/foundation/serialize';

import { inspect } from 'util';

enum RevertCodeEnum {
  OK = 0,
  REVERTED = 1,
}

function isRevertCodeEnum(value: number): value is RevertCodeEnum {
  return value === RevertCodeEnum.OK || value === RevertCodeEnum.REVERTED;
}

/**
 * Wrapper class over a field to safely represent a revert code.
 */
export class RevertCode {
  private code: Fr;
  private constructor(e: RevertCodeEnum) {
    this.code = new Fr(e);
  }

  static readonly OK: RevertCode = new RevertCode(RevertCodeEnum.OK);
  static readonly REVERTED: RevertCode = new RevertCode(RevertCodeEnum.REVERTED);

  public equals(other: RevertCode): boolean {
    return this.code.equals(other.code);
  }

  public isOK(): boolean {
    return this.equals(RevertCode.OK);
  }

  public toBuffer(): Buffer {
    return this.code.toBuffer();
  }

  public toField(): Fr {
    return this.code;
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
    const code = Fr.fromBuffer(reader).toNumber();
    if (!isRevertCodeEnum(code)) {
      throw new Error(`Invalid RevertCode: ${code}`);
    }
    return new RevertCode(code);
  }

  private static readonly NUM_OPTIONS = 2;
  static random(): RevertCode {
    return new RevertCode(Math.floor(Math.random() * RevertCode.NUM_OPTIONS));
  }

  [inspect.custom]() {
    return `RevertCode<${this.code.toString()}>`;
  }
}
