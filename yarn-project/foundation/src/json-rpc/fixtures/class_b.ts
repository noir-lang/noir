/**
 * Test class for testing string converter.
 */
export class ToStringClass {
  constructor(/** A value */ public readonly x: string, /** Another value */ public readonly y: string) {}

  toString(): string {
    return [this.x, this.y].join('-');
  }

  static fromString(value: string) {
    const [x, y] = value.split('-');
    return new ToStringClass(x, y);
  }
}
