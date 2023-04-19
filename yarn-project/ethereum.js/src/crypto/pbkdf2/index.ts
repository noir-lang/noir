import pbkdf2Lib from 'pbkdf2';

/**
 * Derives a cryptographic key from a password using the PBKDF2 (Password-Based Key Derivation Function 2) algorithm.
 * This function returns a Promise that resolves to a Buffer containing the derived key.
 * It uses the 'sha256' hash algorithm as the underlying pseudorandom function.
 *
 * @param password - The input password as a Buffer.
 * @param salt - A unique and random salt value as a Buffer to protect against rainbow table attacks.
 * @param iterations - The number of iterations to perform, which determines the computational cost of the key derivation.
 * @param dklen - The desired length of the derived key in bytes.
 * @returns A Promise that resolves to a Buffer containing the derived key.
 */
export function pbkdf2(password: Buffer, salt: Buffer, iterations: number, dklen: number): Promise<Buffer> {
  return new Promise<Buffer>((resolve, reject) => {
    pbkdf2Lib.pbkdf2(password, salt, iterations, dklen, 'sha256', (err, result) => {
      if (err) {
        reject(err);
      } else {
        resolve(result);
      }
    });
  });
}

/**
 * Synchronously generates a derived key from the given password and salt using the PBKDF2 algorithm with SHA-256.
 * This function is useful when a non-blocking, synchronous operation is required for key derivation.
 *
 * @param password - The input password as a Buffer.
 * @param salt - The salt value as a Buffer.
 * @param iterations - The number of iterations to perform in the PBKDF2 algorithm.
 * @param dklen - The length of the derived key to generate, in bytes.
 * @returns A Buffer containing the derived key.
 */
export function pbkdf2Sync(password: Buffer, salt: Buffer, iterations: number, dklen: number) {
  return pbkdf2Lib.pbkdf2Sync(password, salt, iterations, dklen, 'sha256');
}
