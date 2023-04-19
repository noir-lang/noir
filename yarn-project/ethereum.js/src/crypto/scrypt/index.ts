import { pbkdf2, pbkdf2Sync } from '../pbkdf2/index.js';

const MAX_VALUE = 0x7fffffff;
/**
 * The following is an adaptation of scryptsy.
 *  See: https://www.npmjs.com/package/scryptsy.
 *
 */
function blockmixSalsa8(BY, Yi, r, x, _X) {
  let i;

  arraycopy(BY, (2 * r - 1) * 16, _X, 0, 16);
  for (i = 0; i < 2 * r; i++) {
    blockxor(BY, i * 16, _X, 16);
    salsa208(_X, x);
    arraycopy(_X, 0, BY, Yi + i * 16, 16);
  }

  for (i = 0; i < r; i++) {
    arraycopy(BY, Yi + i * 2 * 16, BY, i * 16, 16);
  }

  for (i = 0; i < r; i++) {
    arraycopy(BY, Yi + (i * 2 + 1) * 16, BY, (i + r) * 16, 16);
  }
}

/**
 * Perform a bitwise rotation operation on the given input values.
 * The value 'a' is left-shifted by 'b' bits, while the remaining rightmost bits are rotated to the left side.
 * This operation is useful in cryptographic algorithms and hash functions.
 *
 * @param a - The initial integer value to be rotated.
 * @param b - The number of bits to perform the rotation operation.
 * @returns The resulting integer value after the bitwise rotation.
 */
function R(a, b) {
  return (a << b) | (a >>> (32 - b));
}

/**
 * Perform the Salsa20/8 core hashing operation on a given input block.
 * This function modifies the provided 512-bit 'B' array in place, applying
 * the Salsa20/8 hash function. The '_X' parameter is used as temporary storage
 * during the computation to avoid additional memory allocations.
 *
 * @param B - The 16-element Uint32Array containing the input block to be hashed.
 * @param _X - A 16-element Uint32Array used as scratch space during computation.
 */
function salsa208(B, x) {
  arraycopy(B, 0, x, 0, 16);

  for (let i = 8; i > 0; i -= 2) {
    x[4] ^= R(x[0] + x[12], 7);
    x[8] ^= R(x[4] + x[0], 9);
    x[12] ^= R(x[8] + x[4], 13);
    x[0] ^= R(x[12] + x[8], 18);
    x[9] ^= R(x[5] + x[1], 7);
    x[13] ^= R(x[9] + x[5], 9);
    x[1] ^= R(x[13] + x[9], 13);
    x[5] ^= R(x[1] + x[13], 18);
    x[14] ^= R(x[10] + x[6], 7);
    x[2] ^= R(x[14] + x[10], 9);
    x[6] ^= R(x[2] + x[14], 13);
    x[10] ^= R(x[6] + x[2], 18);
    x[3] ^= R(x[15] + x[11], 7);
    x[7] ^= R(x[3] + x[15], 9);
    x[11] ^= R(x[7] + x[3], 13);
    x[15] ^= R(x[11] + x[7], 18);
    x[1] ^= R(x[0] + x[3], 7);
    x[2] ^= R(x[1] + x[0], 9);
    x[3] ^= R(x[2] + x[1], 13);
    x[0] ^= R(x[3] + x[2], 18);
    x[6] ^= R(x[5] + x[4], 7);
    x[7] ^= R(x[6] + x[5], 9);
    x[4] ^= R(x[7] + x[6], 13);
    x[5] ^= R(x[4] + x[7], 18);
    x[11] ^= R(x[10] + x[9], 7);
    x[8] ^= R(x[11] + x[10], 9);
    x[9] ^= R(x[8] + x[11], 13);
    x[10] ^= R(x[9] + x[8], 18);
    x[12] ^= R(x[15] + x[14], 7);
    x[13] ^= R(x[12] + x[15], 9);
    x[14] ^= R(x[13] + x[12], 13);
    x[15] ^= R(x[14] + x[13], 18);
  }

  for (let i = 0; i < 16; ++i) {
    B[i] += x[i];
  }
}
/**
 * Naive approach; going back to loop unrolling may yield additional performance.
 */
function blockxor(S, Si, D, len) {
  for (let i = 0; i < len; i++) {
    D[i] ^= S[Si + i];
  }
}

/**
 * Copies elements from the source array to the destination array.
 * Starts copying elements from 'srcPos' index in the source array to 'destPos' index in the destination array, until the given length is reached.
 *
 * @param src - The source array to copy elements from.
 * @param srcPos - The starting position in the source array to begin copying from.
 * @param dest - The destination array to copy elements to.
 * @param destPos - The starting position in the destination array to begin copying to.
 * @param length - The number of elements to be copied from the source to destination array.
 */
function arraycopy(src, srcPos, dest, destPos, length) {
  while (length--) {
    dest[destPos++] = src[srcPos++];
  }
}

/**
 * Ensures the provided value is an integer.
 * Parses the given value to an integer and checks if it's equal to the original value.
 * Throws an error if the parsed and the original values are not equal, indicating that the value is not an integer.
 *
 * @param value - The value to be checked as an integer.
 * @param name - A string representing the name of the parameter, used in the error message if the value is not an integer.
 * @returns The integer value if the value is a valid integer.
 * @throws If the provided value is not an integer.
 */
function ensureInteger(value, name) {
  const intValue = parseInt(value, 10);
  if (value !== intValue) {
    throw new Error('invalid ' + name);
  }
  return intValue;
}
/**
 * N = Cpu cost, r = Memory cost, p = parallelization cost.
 * Callback(error, progress, key).
 */
export function scrypt(password, salt, N, r, p, dkLen, callback?: (progress: number) => boolean) {
  return new Promise<Buffer>((resolve, reject) => {
    N = ensureInteger(N, 'N');
    r = ensureInteger(r, 'r');
    p = ensureInteger(p, 'p');

    dkLen = ensureInteger(dkLen, 'dkLen');

    if (N === 0 || (N & (N - 1)) !== 0) {
      reject(new Error('N must be power of 2'));
    }

    if (N > MAX_VALUE / 128 / r) {
      reject(new Error('N too large'));
    }
    if (r > MAX_VALUE / 128 / p) {
      reject(new Error('r too large'));
    }

    let b = [...pbkdf2Sync(password, salt, 1, p * 128 * r)];
    const B = new Uint32Array(p * 32 * r);
    for (let i = 0; i < B.length; i++) {
      const j = i * 4;
      B[i] =
        ((b[j + 3] & 0xff) << 24) | ((b[j + 2] & 0xff) << 16) | ((b[j + 1] & 0xff) << 8) | ((b[j + 0] & 0xff) << 0);
    }

    const XY = new Uint32Array(64 * r);
    const V = new Uint32Array(32 * r * N);

    const Yi = 32 * r;

    // scratch space
    const x = new Uint32Array(16); // salsa20_8
    const _X = new Uint32Array(16); // blockmix_salsa8

    const totalOps = p * N * 2;
    let currentOp = 0;
    let lastPercent10: any = null;

    // Set this to true to abandon the scrypt on the next step
    let stop = false;

    // State information
    let state = 0;
    let i0 = 0;
    let i1 = 0;
    let Bi;

    // How many blockmix_salsa8 can we do per step?
    const limit = Math.trunc(1000 / r);

    // Trick from scrypt-async; if there is a setImmediate shim in place, use it
    const nextTick: any = typeof setImmediate !== 'undefined' ? setImmediate : setTimeout;

    // This is really all I changed; making scryptsy a state machine so we occasionally
    // stop and give other evnts on the evnt loop a chance to run. ~RicMoo
    const incrementalSMix = async () => {
      if (stop) {
        if (callback) {
          callback(currentOp / totalOps);
        }
        reject(new Error('cancelled'));
        return;
      }

      switch (state) {
        case 0:
          // for (var i = 0; i < p; i++)...
          Bi = i0 * 32 * r;

          arraycopy(B, Bi, XY, 0, Yi); // ROMix - 1

          state = 1; // Move to ROMix 2
          i1 = 0;

        // Fall through

        case 1: {
          // Run up to 1000 steps of the first inner smix loop
          let steps = N - i1;
          if (steps > limit) {
            steps = limit;
          }
          for (let i = 0; i < steps; i++) {
            // ROMix - 2
            arraycopy(XY, 0, V, (i1 + i) * Yi, Yi); // ROMix - 3
            blockmixSalsa8(XY, Yi, r, x, _X); // ROMix - 4
          }

          // for (var i = 0; i < N; i++)
          i1 += steps;
          currentOp += steps;

          // Call the callback with the progress (optionally stopping us)
          const percent10 = Math.trunc((1000 * currentOp) / totalOps);
          if (percent10 !== lastPercent10) {
            if (callback) {
              stop = callback(currentOp / totalOps);
            }
            if (stop) {
              break;
            }
            lastPercent10 = percent10;
          }

          if (i1 < N) {
            break;
          }

          i1 = 0; // Move to ROMix 6
          state = 2;
        }
        // Fall through

        case 2: {
          // Run up to 1000 steps of the second inner smix loop
          let steps = N - i1;
          if (steps > limit) {
            steps = limit;
          }
          for (let i = 0; i < steps; i++) {
            // ROMix - 6
            const offset = (2 * r - 1) * 16; // ROMix - 7
            const j = XY[offset] & (N - 1);
            blockxor(V, j * Yi, XY, Yi); // ROMix - 8 (inner)
            blockmixSalsa8(XY, Yi, r, x, _X); // ROMix - 9 (outer)
          }

          // for (var i = 0; i < N; i++)...
          i1 += steps;
          currentOp += steps;

          // Call the callback with the progress (optionally stopping us)
          const percent10 = Math.trunc((1000 * currentOp) / totalOps);
          if (percent10 !== lastPercent10) {
            if (callback) {
              stop = callback(currentOp / totalOps);
            }
            if (stop) {
              break;
            }
            lastPercent10 = percent10;
          }

          if (i1 < N) {
            break;
          }

          arraycopy(XY, 0, B, Bi, Yi); // ROMix - 10

          // for (var i = 0; i < p; i++)...
          i0++;
          if (i0 < p) {
            state = 0;
            break;
          }

          b = [];
          for (const bb of B) {
            b.push((bb >> 0) & 0xff);
            b.push((bb >> 8) & 0xff);
            b.push((bb >> 16) & 0xff);
            b.push((bb >> 24) & 0xff);
          }

          const derivedKey = await pbkdf2(password, Buffer.from(b), 1, dkLen);

          // Done; don't break (which would reschedule)
          if (callback) {
            callback(1.0);
          }
          resolve(derivedKey);
          return;
        }
      }

      // Schedule the next steps
      nextTick(incrementalSMix);
    };

    // Bootstrap the incremental smix
    void incrementalSMix();
  });
}
