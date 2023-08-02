import request from 'supertest';

import { TestNote, TestState } from '../fixtures/test_state.js';
import { JsonRpcServer } from '../server/index.js';
import { createJsonRpcClient } from './json_rpc_client.js';

test('test an RPC function over client', async () => {
  const mockFetch = async (host: string, method: string, body: any) => {
    const server = new JsonRpcServer(new TestState([new TestNote('a'), new TestNote('b')]), { TestNote }, {}, true);
    const result = await request(server.getApp().callback()).post(`/${method}`).send(body);
    return JSON.parse(result.text);
  };
  const client = createJsonRpcClient<TestState>('', { TestNote }, {}, true, mockFetch);
  const result = await client.addNotes([new TestNote('c')]);
  expect(result[0]).toBeInstanceOf(TestNote);
  expect(result[1]).toBeInstanceOf(TestNote);
  expect(result[2]).toBeInstanceOf(TestNote);
  expect(result[0].toString()).toBe('a');
  expect(result[1].toString()).toBe('b');
  expect(result[2].toString()).toBe('c');
});
