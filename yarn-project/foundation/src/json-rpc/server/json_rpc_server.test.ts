import request from 'supertest';

import { TestNote, TestState } from '../fixtures/test_state.js';
import { JsonRpcServer } from './json_rpc_server.js';

test('test an RPC function with a primitive parameter', async () => {
  const server = new JsonRpcServer(new TestState([new TestNote('a'), new TestNote('b')]), { TestNote }, {}, true);
  const response = await request(server.getApp().callback())
    .post('/getNote')
    .send({ params: [0] });
  expect(response.status).toBe(200);
  expect(response.text).toBe(JSON.stringify({ result: { type: 'TestNote', data: 'a' } }));
});

test('test an RPC function with an array of classes', async () => {
  const server = new JsonRpcServer(new TestState([]), { TestNote }, {}, true);
  const response = await request(server.getApp().callback())
    .post('/addNotes')
    .send({
      params: [[{ data: 'a' }, { data: 'b' }, { data: 'c' }]],
    });
  expect(response.status).toBe(200);
  expect(response.text).toBe(JSON.stringify({ result: [{ data: 'a' }, { data: 'b' }, { data: 'c' }] }));
});

test('test invalid JSON', async () => {
  const server = new JsonRpcServer(new TestState([]), { TestNote }, {}, false);
  const response = await request(server.getApp().callback()).post('/').send('{');
  expect(response.status).toBe(400);
  expect(response.body).toEqual({
    error: { code: -32700, message: 'Parse error' },
    id: null,
    jsonrpc: '2.0',
  });
});

test('invalid method', async () => {
  const server = new JsonRpcServer(new TestState([]), { TestNote }, {}, false);
  const response = await request(server.getApp().callback()).post('/').send({
    jsonrpc: '2.0',
    method: 'invalid',
    params: [],
    id: 42,
  });
  expect(response.status).toBe(400);
  expect(response.body).toEqual({
    error: { code: -32601, message: 'Method not found' },
    id: 42,
    jsonrpc: '2.0',
  });
});
