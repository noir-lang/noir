/**
 * Represents the structure of a JSON-RPC request.
 * Provides a standardized format for remote procedure calls using the JSON data format.
 */
interface JsonRpcRequest {
  /**
   * A JSON-RPC version identifier.
   */
  jsonrpc: string;
  /**
   * The name of the JSON-RPC method to call.
   */
  method: string;
  /**
   * An array of method-specific parameters.
   */
  params: any[];
  /**
   * Unique identifier for the JSON-RPC request.
   */
  id: number;
}

/**
 * Represents a JSON-RPC response object.
 * Provides structured data for handling the result or error from a JSON-RPC call.
 * Used commonly in web3 applications to interact with blockchain networks and services.
 */
interface JsonRpcResponse {
  /**
   * JSON-RPC version used for communication.
   */
  jsonrpc: string;
  /**
   * A unique identifier for the JSON-RPC request.
   */
  id: number;
  /**
   * The outcome of the invoked method.
   */
  result?: any;
  /**
   * Represents error details returned in JSON-RPC response.
   */
  error?: {
    /**
     * The numerical error code representing the type of error occurred.
     */
    code: number;
    /**
     * The name of the method to be called on the remote server.
     */
    message: string;
    /**
     * Additional information related to the error.
     */
    data?: any;
  };
}

/**
 * Type for handling the results and errors of JSON-RPC based Web3 provider send operations.
 */
type Callback = (err?: Error, result?: JsonRpcResponse) => void;

/**
 * Represents a Web3 provider interface for JSON-RPC communication.
 * Provides an abstract method for sending requests to interact with Ethereum blockchain nodes.
 */
export interface Web3Provider {
  send(payload: JsonRpcRequest, callback: Callback): any;
}
