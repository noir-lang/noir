import {
  type AccountWallet,
  type FeePaymentMethod,
  NativeFeePaymentMethod,
  NativeFeePaymentMethodWithClaim,
  NoFeePaymentMethod,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  type SendMethodOptions,
} from '@aztec/aztec.js';
import { AztecAddress, Fr, Gas, GasFees, GasSettings } from '@aztec/circuits.js';
import { type LogFn } from '@aztec/foundation/log';

import { Option } from 'commander';

import { parseBigint } from './utils/commands.js';

export type CliFeeArgs = {
  estimateGasOnly: boolean;
  inclusionFee?: bigint;
  gasLimits?: string;
  payment?: string;
  estimateGas?: boolean;
};

export interface IFeeOpts {
  estimateOnly: boolean;
  gasSettings: GasSettings;
  toSendOpts(sender: AccountWallet): SendMethodOptions;
}

export function printGasEstimates(
  feeOpts: IFeeOpts,
  gasEstimates: Pick<GasSettings, 'gasLimits' | 'teardownGasLimits'>,
  log: LogFn,
) {
  const inclusionFee = feeOpts.gasSettings.inclusionFee;
  log(`Maximum total tx fee:   ${getEstimatedCost(gasEstimates, inclusionFee, GasSettings.default().maxFeesPerGas)}`);
  log(`Estimated total tx fee: ${getEstimatedCost(gasEstimates, inclusionFee, GasFees.default())}`);
  log(`Estimated gas usage:    ${formatGasEstimate(gasEstimates)}`);
}

function formatGasEstimate(estimate: Pick<GasSettings, 'gasLimits' | 'teardownGasLimits'>) {
  return `da=${estimate.gasLimits.daGas},l2=${estimate.gasLimits.l2Gas},teardownDA=${estimate.teardownGasLimits.daGas},teardownL2=${estimate.teardownGasLimits.l2Gas}`;
}

function getEstimatedCost(
  estimate: Pick<GasSettings, 'gasLimits' | 'teardownGasLimits'>,
  inclusionFee: Fr,
  fees: GasFees,
) {
  return GasSettings.from({ ...GasSettings.default(), ...estimate, inclusionFee, maxFeesPerGas: fees })
    .getFeeLimit()
    .toBigInt();
}

export class FeeOpts implements IFeeOpts {
  constructor(
    public estimateOnly: boolean,
    public gasSettings: GasSettings,
    private paymentMethodFactory: (sender: AccountWallet) => FeePaymentMethod,
    private estimateGas: boolean,
  ) {}

  toSendOpts(sender: AccountWallet): SendMethodOptions {
    return {
      estimateGas: this.estimateGas,
      fee: { gasSettings: this.gasSettings ?? GasSettings.default(), paymentMethod: this.paymentMethodFactory(sender) },
    };
  }

  static getOptions() {
    return [
      new Option('--inclusion-fee <value>', 'Inclusion fee to pay for the tx.').argParser(parseBigint),
      new Option('--gas-limits <da=100,l2=100,teardownDA=10,teardownL2=10>', 'Gas limits for the tx.'),
      new Option(
        '--payment <method=name,asset=address,fpc=address,claimSecret=string,claimAmount=string,rebateSecret=string>',
        'Fee payment method and arguments. Valid methods are: none, native, fpc-public, fpc-private.',
      ),
      new Option('--no-estimate-gas', 'Whether to automatically estimate gas limits for the tx.'),
      new Option('--estimate-gas-only', 'Only report gas estimation for the tx, do not send it.'),
    ];
  }

  static fromCli(args: CliFeeArgs, log: LogFn) {
    const estimateOnly = args.estimateGasOnly;
    if (!args.inclusionFee && !args.gasLimits && !args.payment) {
      return new NoFeeOpts(estimateOnly);
    }
    const gasSettings = GasSettings.from({
      ...GasSettings.default(),
      ...(args.gasLimits ? parseGasLimits(args.gasLimits) : {}),
      ...(args.inclusionFee ? { inclusionFee: new Fr(args.inclusionFee) } : {}),
      maxFeesPerGas: GasFees.default(),
    });
    return new FeeOpts(
      estimateOnly,
      gasSettings,
      args.payment ? parsePaymentMethod(args.payment, log) : () => new NoFeePaymentMethod(),
      !!args.estimateGas,
    );
  }
}

class NoFeeOpts implements IFeeOpts {
  constructor(public estimateOnly: boolean) {}

  get gasSettings(): GasSettings {
    return GasSettings.default();
  }

  toSendOpts(): SendMethodOptions {
    return {};
  }
}

function parsePaymentMethod(payment: string, log: LogFn): (sender: AccountWallet) => FeePaymentMethod {
  const parsed = payment.split(',').reduce((acc, item) => {
    const [dimension, value] = item.split('=');
    acc[dimension] = value;
    return acc;
  }, {} as Record<string, string>);

  const getFpcOpts = (parsed: Record<string, string>) => {
    if (!parsed.fpc) {
      throw new Error('Missing "fpc" in payment option');
    }
    if (!parsed.asset) {
      throw new Error('Missing "asset" in payment option');
    }

    return [AztecAddress.fromString(parsed.asset), AztecAddress.fromString(parsed.fpc)];
  };

  return (sender: AccountWallet) => {
    switch (parsed.method) {
      case 'none':
        log('Using no fee payment');
        return new NoFeePaymentMethod();
      case 'native':
        if (parsed.claimSecret && parsed.claimAmount) {
          log(`Using native fee payment method with claim for ${parsed.claimAmount} tokens`);
          return new NativeFeePaymentMethodWithClaim(
            sender.getAddress(),
            BigInt(parsed.claimAmount),
            Fr.fromString(parsed.claimSecret),
          );
        } else {
          log(`Using native fee payment`);
          return new NativeFeePaymentMethod(sender.getAddress());
        }
      case 'fpc-public': {
        const [asset, fpc] = getFpcOpts(parsed);
        log(`Using public fee payment with asset ${asset} via paymaster ${fpc}`);
        return new PublicFeePaymentMethod(asset, fpc, sender);
      }
      case 'fpc-private': {
        const [asset, fpc] = getFpcOpts(parsed);
        const rebateSecret = parsed.rebateSecret ? Fr.fromString(parsed.rebateSecret) : Fr.random();
        log(
          `Using private fee payment with asset ${asset} via paymaster ${fpc} with rebate secret ${rebateSecret.toString()}`,
        );
        return new PrivateFeePaymentMethod(asset, fpc, sender, rebateSecret);
      }
      case undefined:
        throw new Error('Missing "method" in payment option');
      default:
        throw new Error(`Invalid fee payment method: ${payment}`);
    }
  };
}

function parseGasLimits(gasLimits: string): { gasLimits: Gas; teardownGasLimits: Gas } {
  const parsed = gasLimits.split(',').reduce((acc, limit) => {
    const [dimension, value] = limit.split('=');
    acc[dimension] = parseInt(value, 10);
    return acc;
  }, {} as Record<string, number>);

  const expected = ['da', 'l2', 'teardownDA', 'teardownL2'];
  for (const dimension of expected) {
    if (!(dimension in parsed)) {
      throw new Error(`Missing gas limit for ${dimension}`);
    }
  }

  return {
    gasLimits: new Gas(parsed.da, parsed.l2),
    teardownGasLimits: new Gas(parsed.teardownDA, parsed.teardownL2),
  };
}
