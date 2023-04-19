/**
 * Represents the typed data structure used in EIP-712 for signing messages.
 * Provides a consistent format to create domain, message types and message values
 * that can be encoded and signed, improving security and user experience in blockchain transactions.
 */
export interface TypedData {
  /**
   * Represents EIP-712 domain data containing application-specific information for signing typed data.
   */
  domain: {
    /**
     * The identifying name of the domain.
     */
    name: string;
    /**
     * Version identifier for the domain.
     */
    version: string;
    /**
     * The unique identifier of the blockchain network.
     */
    chainId: number;
    /**
     * The address of the contract responsible for data verification.
     */
    verifyingContract: string;
  };
  /**
   * An object containing structured data types for EIP-712 signing.
   */
  types: {
    [key: string]: {
      /**
       * The name of the domain in which the TypedData is structured.
       */
      name: string;
      /**
       * A mapping of data types with their corresponding properties, including name and type.
       */
      type: string;
    }[];
  };
  /**
   * The specific structured data to be signed and verified.
   */
  message: any;
  /**
   * The main type used for structuring and verifying the EIP-712 typed data.
   */
  primaryType: string;
}
