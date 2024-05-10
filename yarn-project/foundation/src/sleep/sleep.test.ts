import { jest } from '@jest/globals';

import { InterruptError } from '../error/index.js';
import { InterruptibleSleep } from './index.js';

describe('InterruptibleSleep', () => {
  it('should sleep for 100ms', async () => {
    const sleeper = new InterruptibleSleep();
    const start = Date.now();
    await sleeper.sleep(100);
    const end = Date.now();
    // -1 ms wiggle room for rounding errors
    expect(end - start).toBeGreaterThanOrEqual(99);
  });

  it('can start multiple sleeps', async () => {
    const sleeper = new InterruptibleSleep();
    const start = Date.now();
    await Promise.all([sleeper.sleep(100), sleeper.sleep(150)]);
    const end = Date.now();
    expect(end - start).toBeGreaterThanOrEqual(149);
  });

  it('can interrupt multiple sleeps', async () => {
    const stub = jest.fn();
    const sleeper = new InterruptibleSleep();
    const start = Date.now();
    let end1;
    const sleep1 = sleeper.sleep(100).then(() => {
      end1 = Date.now();
    });
    const sleep2 = sleeper.sleep(150).then(stub);
    setTimeout(() => sleeper.interrupt(true), 125);
    await Promise.all([sleep1, sleep2]).catch(e => expect(e).toBeInstanceOf(InterruptError));
    expect(end1! - start).toBeGreaterThanOrEqual(99);
    expect(stub).not.toHaveBeenCalled();
  });
});
