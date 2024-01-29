import request from 'supertest';

import { TestNote, TestState } from '../fixtures/test_state.js';
import { JsonRpcServer, createNamespacedJsonRpcServer } from '../server/index.js';
import { createJsonRpcClient } from './json_rpc_client.js';

it('test an RPC function over client', async () => {
  const mockFetch = async (host: string, method: string, body: any) => {
    const server = new JsonRpcServer(new TestState([new TestNote('a'), new TestNote('b')]), { TestNote }, {});
    const result = await request(server.getApp().callback()).post(`/`).send(body);
    return JSON.parse(result.text);
  };
  const client = createJsonRpcClient<TestState>('', { TestNote }, {}, true, false, mockFetch);
  const result = await client.addNotes([new TestNote('c')]);
  expect(result[0]).toBeInstanceOf(TestNote);
  expect(result[1]).toBeInstanceOf(TestNote);
  expect(result[2]).toBeInstanceOf(TestNote);
  expect(result[0].toString()).toBe('a');
  expect(result[1].toString()).toBe('b');
  expect(result[2].toString()).toBe('c');
});

it('test a namespaced RPC function over client', async () => {
  const namespace = 'testService';
  const mockFetch = async (host: string, method: string, body: any) => {
    const service = new JsonRpcServer(new TestState([new TestNote('a'), new TestNote('b')]), { TestNote }, {});
    const server = createNamespacedJsonRpcServer([{ [namespace]: service }]);
    const result = await request(server.getApp().callback()).post('/').send(body);
    return JSON.parse(result.text);
  };
  const client = createJsonRpcClient<TestState>('', { TestNote }, {}, true, namespace, mockFetch);
  const result = await client.addNotes([new TestNote('c')]);
  expect(result[0]).toBeInstanceOf(TestNote);
  expect(result[1]).toBeInstanceOf(TestNote);
  expect(result[2]).toBeInstanceOf(TestNote);
  expect(result[0].toString()).toBe('a');
  expect(result[1].toString()).toBe('b');
  expect(result[2].toString()).toBe('c');
});
