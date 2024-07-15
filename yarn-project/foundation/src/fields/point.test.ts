import { Fr } from './fields.js';
import { Point } from './point.js';

describe('Point', () => {
  it('converts to and from x and sign of y coordinate', () => {
    const p = new Point(
      new Fr(0x30426e64aee30e998c13c8ceecda3a77807dbead52bc2f3bf0eae851b4b710c1n),
      new Fr(0x113156a068f603023240c96b4da5474667db3b8711c521c748212a15bc034ea6n),
      false,
    );

    const [x, sign] = p.toXAndSign();
    const p2 = Point.fromXAndSign(x, sign);

    expect(p.equals(p2)).toBeTruthy();
  });

  it('creates a valid random point', () => {
    expect(Point.random().isOnGrumpkin()).toBeTruthy();
  });

  it('converts to and from buffer', () => {
    const p = Point.random();
    const p2 = Point.fromBuffer(p.toBuffer());

    expect(p.equals(p2)).toBeTruthy();
  });

  it('converts to and from compressed buffer', () => {
    const p = Point.random();
    const p2 = Point.fromCompressedBuffer(p.toCompressedBuffer());

    expect(p.equals(p2)).toBeTruthy();
  });
});
