/**
 * ProviderError is an enumeration representing specific error codes related to Ethereum provider communication.
 * These error codes can be used by applications to handle various provider-related events or issues, such as user rejection, unauthorized access, unsupported functionality, and connection problems.
 * By standardizing these error codes, it allows for more consistent and robust error handling across different Ethereum providers and applications.
 */
export enum ProviderError {
  USER_REJECTED = 4001,
  UNAUTHORIZED = 4100,
  UNSUPPORTED = 4200,
  DISCONNECTED = 4900,
  CHAIN_DISCONNECTED = 4901,
}

/**
 * Represents a standardized message format for communication between Ethereum providers and applications.
 * Contains type information to identify the purpose of the message and data payload for further processing.
 */
export interface ProviderMessage {
  /**
   * The type of provider notification event.
   */
  readonly type: string;
  /**
   * Arbitrary data associated with the provider message.
   */
  readonly data: unknown;
}

/**
 * Represents the arguments for an Ethereum RPC request.
 * Provides the necessary method and optional parameters to form a well-structured request to interact with the Ethereum network.
 */
export interface RequestArguments {
  /**
   * The JSON-RPC method to be called.
   */
  readonly method: string;
  /**
   * An optional array of method-specific parameters.
   */
  readonly params?: any[];
}

/**
 * Represents an error encountered during Provider's RPC communication.
 * It extends the native Error object and includes additional properties
 * such as error code and data, providing more context about the occurred error.
 */
export interface ProviderRpcError extends Error {
  /**
   * Represents a provider-specific message, typically used for communicating events or updates.
   */
  message: string;
  /**
   * The error code representing the type of provider error.
   */
  code: ProviderError | number;
  /**
   * An arbitrary data payload related to the corresponding provider event or error.
   */
  data?: unknown;
}

/**
 * Represents the connection information for an Ethereum provider.
 * Provides details such as the chain ID to ensure compatibility and connectivity with the desired blockchain network.
 */
export interface ProviderConnectInfo {
  /**
   * The unique identifier for the connected blockchain network.
   */
  readonly chainId: string;
}

/**
 * Type representing the different types of notifications that an EthereumProvider can emit.
 * These notifications are related to events such as connection, disconnection, chain and account changes, and incoming messages.
 */
export type EthereumProviderNotifications = 'connect' | 'disconnect' | 'chainChanged' | 'accountsChanged' | 'message';

/**
 * Interface defining an EIP1193 compatible provider. This is the standard that all future providers should adhere to.
 * The Aztec SDK accepts such a provider. If non standard providers wish to be used, wrap them in an adapter first.
 * Two adapters are provided, an EthersAdapter for ethers providers, and Web3Adapter, for legacy web3 providers.
 */
export interface EthereumProvider {
  request(args: RequestArguments): Promise<any>;

  on(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
  on(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
  on(notification: 'chainChanged', listener: (chainId: string) => void): this;
  on(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
  on(notification: 'message', listener: (message: ProviderMessage) => void): this;

  removeListener(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
  removeListener(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
  removeListener(notification: 'chainChanged', listener: (chainId: string) => void): this;
  removeListener(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
  removeListener(notification: 'message', listener: (message: ProviderMessage) => void): this;
}
