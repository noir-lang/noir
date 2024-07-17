import { updateInlineTestData } from '../testing/test_data.js';
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

  it('compressed point with + sign matches Noir', () => {
    const p = new Point(
      new Fr(0x1af41f5de96446dc3776a1eb2d98bb956b7acd9979a67854bec6fa7c2973bd73n),
      new Fr(0x07fc22c7f2c7057571f137fe46ea9c95114282bc95d37d71ec4bfb88de457d4an),
      false,
    );
    expect(p.toXAndSign()[1]).toBe(true);

    const compressed = p.toCompressedBuffer().toString('hex');
    expect(compressed).toMatchInlineSnapshot(`"9af41f5de96446dc3776a1eb2d98bb956b7acd9979a67854bec6fa7c2973bd73"`);

    const byteArrayString = `[${compressed.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))}]`;

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/utils/point.nr',
      'expected_compressed_point_positive_sign',
      byteArrayString,
    );
  });

  it('compressed point with - sign matches Noir', () => {
    const p = new Point(
      new Fr(0x247371652e55dd74c9af8dbe9fb44931ba29a9229994384bd7077796c14ee2b5n),
      new Fr(0x26441aec112e1ae4cee374f42556932001507ad46e255ffb27369c7e3766e5c0n),
      false,
    );
    expect(p.toXAndSign()[1]).toBe(false);

    const compressed = p.toCompressedBuffer().toString('hex');
    expect(compressed).toMatchInlineSnapshot(`"247371652e55dd74c9af8dbe9fb44931ba29a9229994384bd7077796c14ee2b5"`);

    const byteArrayString = `[${compressed.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))}]`;

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/utils/point.nr',
      'expected_compressed_point_negative_sign',
      byteArrayString,
    );
  });
});
