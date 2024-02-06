// TODO(#4411): this is mostly redundant. Nuke this
import {
  makeCallRequest,
  makeFinalAccumulatedData,
  makeNewSideEffect,
  makePreviousKernelData,
  makePrivateCallData,
  makePrivateKernelInputsInner,
  makePrivateKernelPublicInputsFinal,
} from '../../tests/factories.js';
import {
  CallRequest,
  FinalAccumulatedData,
  PreviousKernelData,
  PrivateKernelPublicInputsFinal,
  SideEffect,
} from '../index.js';
import { PrivateCallData, PrivateKernelInputsInner } from './private_kernel.js';

describe('PrivateKernel', function () {
  it(`serializes PrivateKernelInputsInner to buffer and deserializes it back`, () => {
    const expected = makePrivateKernelInputsInner();
    const buffer = expected.toBuffer();
    const res = PrivateKernelInputsInner.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});

describe('Public call request', () => {
  it('convert to and from buffer', () => {
    const pkpi = makePrivateKernelPublicInputsFinal();
    const buf = pkpi.toBuffer();
    expect(PrivateKernelPublicInputsFinal.fromBuffer(buf)).toEqual(pkpi);
  });
});

describe('SideEffect', () => {
  it('convert to and from buffer', () => {
    const sde = makeNewSideEffect(0);
    const buf = sde.toBuffer();
    expect(SideEffect.fromBuffer(buf)).toEqual(sde);
  });
});

describe('FinalAccumulatedData', () => {
  it('convert to and from buffer', () => {
    const fad = makeFinalAccumulatedData(0);
    const buf = fad.toBuffer();
    expect(FinalAccumulatedData.fromBuffer(buf)).toEqual(fad);
  });
});

describe('PreviousKernelData', () => {
  it('convert to and from buffer', () => {
    const fad = makePreviousKernelData(0);
    const buf = fad.toBuffer();
    expect(PreviousKernelData.fromBuffer(buf)).toEqual(fad);
  });
});

describe('PrivateCallData', () => {
  it('convert to and from buffer', () => {
    const fad = makePrivateCallData(0);
    const buf = fad.toBuffer();
    expect(PrivateCallData.fromBuffer(buf)).toEqual(fad);
  });
});

describe('CallRequest', () => {
  it('convert to and from buffer', () => {
    const fad = makeCallRequest(0);
    const buf = fad.toBuffer();
    expect(CallRequest.fromBuffer(buf)).toEqual(fad);
  });
});
