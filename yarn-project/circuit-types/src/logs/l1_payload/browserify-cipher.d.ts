declare module 'browserify-cipher' {
  import { type Cipher } from 'crypto';

  /**
   * Type representing supported cipher algorithms for encryption and decryption.
   */
  type CipherTypes = 'aes-128-cbc';

  /**
   * Represents the options for cipher operations.
   * Provides optional configuration settings to customize cipher behavior during encryption and decryption processes.
   */
  interface CipherOptions {
    /**
     * Initialization vector used for encryption/decryption process.
     */
    iv?: Buffer;
  }

  /**
   * Create a Cipher instance using the specified algorithm, key, and optional options.
   * The function supports a limited set of algorithms defined by CipherTypes, such as 'aes-128-cbc'.
   * The 'key' parameter must be a Buffer containing the secret key for encryption.
   * The optional 'options' parameter can include an initialization vector (iv) as a Buffer.
   * Throws an error if the specified algorithm is not supported or the provided key/iv are invalid.
   *
   * @param algorithm - The encryption algorithm to be used, as defined in CipherTypes.
   * @param key - A Buffer containing the secret key for encryption.
   * @param options - Optional configuration object with properties like 'iv' for initialization vector.
   * @returns A Cipher instance configured with the specified algorithm, key, and options.
   */
  function createCipher(algorithm: CipherTypes, key: Buffer, options?: CipherOptions): Cipher;
  /**
   * Create a Cipher instance with an explicit initialization vector (IV) for the specified algorithm and key.
   * The 'algorithm' should be one of the supported cipher types, such as 'aes-128-cbc'.
   * The 'key' and 'iv' must be provided as Buffers. The IV length should match the block size of the chosen algorithm.
   * Throws an error if the provided algorithm is not supported, or the key or IV lengths are invalid.
   *
   * @param algorithm - The cipher algorithm to be used, such as 'aes-128-cbc'.
   * @param key - A Buffer containing the encryption key.
   * @param iv - A Buffer containing the initialization vector.
   * @returns A Cipher instance initialized with the specified algorithm, key, and IV.
   */
  function createCipheriv(algorithm: CipherTypes, key: Buffer, iv: Buffer): Cipher;
  /**
   * Create a Decipher object for the given algorithm and key, which can be used to decrypt data.
   * The 'algorithm' must be one of the supported cipher types (e.g., 'aes-128-cbc').
   * The 'key' should be a Buffer containing the secret key for decryption.
   * An optional 'options' object can be provided to specify additional properties such as IV (initialization vector).
   * Throws an error if the inputs are invalid or the specified algorithm is not supported.
   *
   * @param algorithm - The cipher type to be used for decryption.
   * @param key - A Buffer containing the secret key for decryption.
   * @param options - An optional CipherOptions object with additional properties.
   * @returns A Decipher object that can be used to decrypt data.
   */
  function createDecipher(algorithm: CipherTypes, key: Buffer, options?: CipherOptions): Cipher;
  /**
   * Create a decipher object using the specified algorithm, key, and initialization vector (iv).
   * The function allows for creating a custom decryption stream with the specific algorithm
   * and provided parameters. It is useful for decrypting data that was encrypted with a custom
   * initialization vector. Throws an error if the algorithm is not supported or invalid parameters are provided.
   *
   * @param algorithm - The encryption algorithm to be used, e.g., 'aes-128-cbc'.
   * @param key - The encryption key in the form of a Buffer.
   * @param iv - The initialization vector as a Buffer.
   * @returns A Decipher object which can be used to decrypt data.
   */
  function createDecipheriv(algorithm: CipherTypes, key: Buffer, iv: Buffer): Cipher;
  /**
   * Retrieves the list of supported cipher algorithms.
   * This function returns an array of strings containing the names of all currently available
   * cipher algorithms that can be used for encryption and decryption operations.
   *
   * @returns An array of strings representing the supported cipher algorithms.
   */
  function getCiphers(): CipherTypes[];
}
