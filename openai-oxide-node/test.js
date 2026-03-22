const assert = require('node:assert/strict')

process.env.OPENAI_API_KEY ??= 'test-key'

const bindings = require('./index.js')

assert.equal(typeof bindings.Client, 'function', 'Client export is missing')
assert.equal(typeof bindings.NodeWsSession, 'function', 'NodeWsSession export is missing')

const client = new bindings.Client()

assert.equal(typeof client.createChatCompletion, 'function')
assert.equal(typeof client.createChatStream, 'function')
assert.equal(typeof client.createResponse, 'function')
assert.equal(typeof client.createStream, 'function')
assert.equal(typeof client.wsSession, 'function')

console.log('openai-oxide-node smoke test passed')
