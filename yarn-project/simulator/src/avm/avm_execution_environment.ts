import { FunctionSelector, type GasSettings, type GlobalVariables, type Header } from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

export class AvmContextInputs {
  static readonly SIZE = 3;

  constructor(private selector: Fr, private argsHash: Fr, private isStaticCall: boolean) {}

  public toFields(): Fr[] {
    return [this.selector, this.argsHash, new Fr(this.isStaticCall)];
  }
}

/**
 * Contains variables that remain constant during AVM execution
 * These variables are provided by the public kernel circuit
 */
export class AvmExecutionEnvironment {
  private readonly calldataPrefixLength;
  constructor(
    public readonly address: AztecAddress,
    public readonly storageAddress: AztecAddress,
    public readonly sender: AztecAddress,
    public readonly feePerL2Gas: Fr,
    public readonly feePerDaGas: Fr,
    public readonly contractCallDepth: Fr,
    public readonly header: Header,
    public readonly globals: GlobalVariables,
    public readonly isStaticCall: boolean,
    public readonly isDelegateCall: boolean,
    public readonly calldata: Fr[],
    public readonly gasSettings: GasSettings,
    public readonly transactionFee: Fr,

    // Function selector may be temporary since eventually public contract bytecode will likely be one
    // blob containing all functions, and function selector will become an application-level mechanism
    // (e.g. first few bytes of calldata + compiler-generated jump table)
    public readonly functionSelector: FunctionSelector,
  ) {
    // We encode some extra inputs (AvmContextInputs) in calldata.
    // This will have to go once we move away from one proof per call.
    const inputs = new AvmContextInputs(
      functionSelector.toField(),
      computeVarArgsHash(calldata),
      isStaticCall,
    ).toFields();
    this.calldata = [...inputs, ...calldata];
    this.calldataPrefixLength = inputs.length;
  }

  private deriveEnvironmentForNestedCallInternal(
    targetAddress: AztecAddress,
    calldata: Fr[],
    functionSelector: FunctionSelector,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    return new AvmExecutionEnvironment(
      /*address=*/ targetAddress,
      /*storageAddress=*/ targetAddress,
      /*sender=*/ this.address,
      this.feePerL2Gas,
      this.feePerDaGas,
      this.contractCallDepth.add(Fr.ONE),
      this.header,
      this.globals,
      isStaticCall,
      isDelegateCall,
      calldata,
      this.gasSettings,
      this.transactionFee,
      functionSelector,
    );
  }

  public deriveEnvironmentForNestedCall(
    targetAddress: AztecAddress,
    calldata: Fr[],
    functionSelector: FunctionSelector = FunctionSelector.empty(),
  ): AvmExecutionEnvironment {
    return this.deriveEnvironmentForNestedCallInternal(
      targetAddress,
      calldata,
      functionSelector,
      /*isStaticCall=*/ false,
      /*isDelegateCall=*/ false,
    );
  }

  public deriveEnvironmentForNestedStaticCall(
    targetAddress: AztecAddress,
    calldata: Fr[],
    functionSelector: FunctionSelector,
  ): AvmExecutionEnvironment {
    return this.deriveEnvironmentForNestedCallInternal(
      targetAddress,
      calldata,
      functionSelector,
      /*isStaticCall=*/ true,
      /*isDelegateCall=*/ false,
    );
  }

  public newDelegateCall(
    _targetAddress: AztecAddress,
    _calldata: Fr[],
    _functionSelector: FunctionSelector,
  ): AvmExecutionEnvironment {
    throw new Error('Delegate calls not supported!');
  }

  public getCalldataWithoutPrefix(): Fr[] {
    // clip off the first few entries
    return this.calldata.slice(this.calldataPrefixLength);
  }
}
