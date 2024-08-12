import {
  type AccountWallet,
  FeeJuicePaymentMethod,
  FeeJuicePaymentMethodWithClaim,
  type FeePaymentMethod,
  NoFeePaymentMethod,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  type SendMethodOptions,
} from '@aztec/aztec.js';
import { AztecAddress, Fr, Gas, GasFees, GasSettings } from '@aztec/circuits.js';
import { parseBigint } from '@aztec/cli/utils';
import { type LogFn } from '@aztec/foundation/log';

import { Option } from 'commander';

import { type WalletDB } from '../../storage/wallet_db.js';
import { aliasedAddressParser } from './index.js';

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
  toSendOpts(sender: AccountWallet): Promise<SendMethodOptions>;
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
    private paymentMethodFactory: (sender: AccountWallet) => Promise<FeePaymentMethod>,
    private estimateGas: boolean,
  ) {}

  async toSendOpts(sender: AccountWallet): Promise<SendMethodOptions> {
    return {
      estimateGas: this.estimateGas,
      fee: {
        gasSettings: this.gasSettings ?? GasSettings.default(),
        paymentMethod: await this.paymentMethodFactory(sender),
      },
    };
  }

  static getOptions() {
    return [
      new Option('--inclusion-fee <value>', 'Inclusion fee to pay for the tx.').argParser(parseBigint),
      new Option('--gas-limits <da=100,l2=100,teardownDA=10,teardownL2=10>', 'Gas limits for the tx.'),
      new Option(
        '--payment <method=name,asset=address,fpc=address,claimSecret=string,claimAmount=string,rebateSecret=string>',
        'Fee payment method and arguments. Valid methods are: none, fee_juice, fpc-public, fpc-private.',
      ),
      new Option('--no-estimate-gas', 'Whether to automatically estimate gas limits for the tx.'),
      new Option('--estimate-gas-only', 'Only report gas estimation for the tx, do not send it.'),
    ];
  }

  static fromCli(args: CliFeeArgs, log: LogFn, db?: WalletDB) {
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
      args.payment ? parsePaymentMethod(args.payment, log, db) : () => Promise.resolve(new NoFeePaymentMethod()),
      !!args.estimateGas,
    );
  }
}

class NoFeeOpts implements IFeeOpts {
  constructor(public estimateOnly: boolean) {}

  get gasSettings(): GasSettings {
    return GasSettings.default();
  }

  toSendOpts(): Promise<SendMethodOptions> {
    return Promise.resolve({});
  }
}

function parsePaymentMethod(
  payment: string,
  log: LogFn,
  db?: WalletDB,
): (sender: AccountWallet) => Promise<FeePaymentMethod> {
  const parsed = payment.split(',').reduce((acc, item) => {
    const [dimension, value] = item.split('=');
    acc[dimension] = value ?? 1;
    return acc;
  }, {} as Record<string, string>);

  const getFpcOpts = (parsed: Record<string, string>, db?: WalletDB) => {
    if (!parsed.fpc) {
      throw new Error('Missing "fpc" in payment option');
    }
    if (!parsed.asset) {
      throw new Error('Missing "asset" in payment option');
    }

    const fpc = aliasedAddressParser('contracts', parsed.fpc, db);

    return [AztecAddress.fromString(parsed.asset), fpc];
  };

  return async (sender: AccountWallet) => {
    switch (parsed.method) {
      case 'none':
        log('Using no fee payment');
        return new NoFeePaymentMethod();
      case 'native':
        if (parsed.claim || (parsed.claimSecret && parsed.claimAmount)) {
          let claimAmount, claimSecret;
          if (parsed.claim && db) {
            ({ amount: claimAmount, secret: claimSecret } = await db.popBridgedFeeJuice(sender.getAddress(), log));
          } else {
            ({ claimAmount, claimSecret } = parsed);
          }
          log(`Using Fee Juice for fee payments with claim for ${parsed.claimAmount} tokens`);
          return new FeeJuicePaymentMethodWithClaim(
            sender.getAddress(),
            BigInt(claimAmount),
            Fr.fromString(claimSecret),
          );
        } else {
          log(`Using Fee Juice for fee payment`);
          return new FeeJuicePaymentMethod(sender.getAddress());
        }
      case 'fpc-public': {
        const [asset, fpc] = getFpcOpts(parsed, db);
        log(`Using public fee payment with asset ${asset} via paymaster ${fpc}`);
        return new PublicFeePaymentMethod(asset, fpc, sender);
      }
      case 'fpc-private': {
        const [asset, fpc] = getFpcOpts(parsed, db);
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
