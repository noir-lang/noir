import request from 'supertest';
import { TestState, TestNote } from '../fixtures/test_state.js';
import { JsonRpcServer } from './json_rpc_server.js';

test('test an RPC function with a primitive parameter', async () => {
  const server = new JsonRpcServer(new TestState([new TestNote('a'), new TestNote('b')]), { TestNote });
  const response = await request(server.getApp().callback())
    .post('/getNote')
    .send({ params: [0] });
  expect(response.status).toBe(200);
  expect(response.text).toBe('{"result":{"type":"TestNote","data":"a"}}');
});

test('test an RPC function with an array of classes', async () => {
  const server = new JsonRpcServer(new TestState([]), { TN: TestNote });
  const response = await request(server.getApp().callback())
    .post('/addNotes')
    .send({
      params: [
        [
          { type: 'TN', data: 'a' },
          { type: 'TN', data: 'b' },
          { type: 'TN', data: 'c' },
        ],
      ],
    });
  expect(response.status).toBe(200);
  expect(response.text).toBe('{"result":[{"type":"TN","data":"a"},{"type":"TN","data":"b"},{"type":"TN","data":"c"}]}');
});
