/**
 * Drop-in compatibility layer matching the official `openai` npm package.
 *
 * Change one import:
 *   // Before (official SDK):
 *   import OpenAI from 'openai';
 *
 *   // After (openai-oxide — Rust-powered, faster):
 *   const { OpenAI } = require('openai-oxide/compat');
 *
 *   // Same code works:
 *   const client = new OpenAI();
 *   const r = await client.chat.completions.create({ model: 'gpt-5.4-mini', messages: [...] });
 *   console.log(r.choices[0].message.content);
 */

const { Client } = require('./index');

class Completions {
  constructor(native) { this._native = native; }

  /** POST /chat/completions */
  async create(params) {
    if (params.stream) {
      return this._createStream(params);
    }
    return this._native.createChatCompletion(params);
  }

  /** POST /chat/completions with response_format auto-set from Zod/JSON schema */
  async parse(params) {
    const { response_format, ...rest } = params;
    if (response_format && typeof response_format.parse === 'function') {
      // Zod schema — need zod-to-json-schema
      let zodToJsonSchema;
      try { zodToJsonSchema = require('zod-to-json-schema').zodToJsonSchema; }
      catch { throw new Error('Install zod-to-json-schema for Zod support'); }
      const schema = zodToJsonSchema(response_format);
      const name = response_format.description || 'Response';
      const result = await this._native.createChatParsed(rest, name, schema);
      return result;
    }
    if (response_format && response_format.type === 'json_schema') {
      // Already a JSON schema — pass through
      return this._native.createChatCompletion({ ...rest, response_format });
    }
    return this._native.createChatCompletion(params);
  }

  /** Streaming — returns async iterator */
  _createStream(params) {
    const native = this._native;
    return new Promise((resolve, reject) => {
      const chunks = [];
      let done = false;
      const iterator = {
        [Symbol.asyncIterator]() { return this; },
        next() {
          return new Promise((res) => {
            if (chunks.length > 0) return res({ value: chunks.shift(), done: false });
            if (done) return res({ value: undefined, done: true });
            // Wait for next chunk
            iterator._resolve = res;
          });
        },
        _resolve: null,
        _push(chunk) {
          if (iterator._resolve) {
            const r = iterator._resolve;
            iterator._resolve = null;
            r({ value: chunk, done: false });
          } else {
            chunks.push(chunk);
          }
        },
        _end() {
          done = true;
          if (iterator._resolve) {
            const r = iterator._resolve;
            iterator._resolve = null;
            r({ value: undefined, done: true });
          }
        }
      };

      native.createChatStream(params, (err, event) => {
        if (err) {
          if (iterator._resolve) {
            const r = iterator._resolve;
            iterator._resolve = null;
            r({ value: undefined, done: true });
          }
          return;
        }
        if (event && event.type === 'done') {
          iterator._end();
        } else if (event) {
          iterator._push(event);
        }
      }).then(() => iterator._end()).catch(reject);

      resolve(iterator);
    });
  }
}

class Chat {
  constructor(native) {
    this.completions = new Completions(native);
  }
}

class Responses {
  constructor(native) { this._native = native; }

  /** POST /responses */
  async create(params) {
    if (params.stream) {
      return this._createStream(params);
    }
    return this._native.createResponse(params);
  }

  /** POST /responses with structured output */
  async parse(params) {
    const { text_format, text, ...rest } = params;
    if (text_format) {
      // Zod or JSON schema for text.format
      let schema, name;
      if (typeof text_format.parse === 'function') {
        let zodToJsonSchema;
        try { zodToJsonSchema = require('zod-to-json-schema').zodToJsonSchema; }
        catch { throw new Error('Install zod-to-json-schema for Zod support'); }
        schema = zodToJsonSchema(text_format);
        name = text_format.description || 'Response';
      } else {
        schema = text_format;
        name = text_format.name || 'Response';
      }
      return this._native.createResponseParsed(rest, name, schema);
    }
    return this._native.createResponse(params);
  }

  _createStream(params) {
    const native = this._native;
    return new Promise((resolve, reject) => {
      const chunks = [];
      let done = false;
      const iterator = {
        [Symbol.asyncIterator]() { return this; },
        next() {
          return new Promise((res) => {
            if (chunks.length > 0) return res({ value: chunks.shift(), done: false });
            if (done) return res({ value: undefined, done: true });
            iterator._resolve = res;
          });
        },
        _resolve: null,
        _push(chunk) {
          if (iterator._resolve) {
            const r = iterator._resolve;
            iterator._resolve = null;
            r({ value: chunk, done: false });
          } else {
            chunks.push(chunk);
          }
        },
        _end() {
          done = true;
          if (iterator._resolve) {
            const r = iterator._resolve;
            iterator._resolve = null;
            r({ value: undefined, done: true });
          }
        }
      };

      native.createStream(params, (err, event) => {
        if (err) { iterator._end(); return; }
        if (event && event.type === 'done') { iterator._end(); }
        else if (event) { iterator._push(event); }
      }).then(() => iterator._end()).catch(reject);

      resolve(iterator);
    });
  }
}

class Embeddings {
  constructor(native) { this._native = native; }
  async create(params) {
    return this._native.createChatCompletion({ ...params, _endpoint: 'embeddings' });
  }
}

class Models {
  constructor(native) { this._native = native; }
  async list() {
    return JSON.parse(await this._native.createChatCompletion({ _endpoint: 'models' }));
  }
}

/**
 * Drop-in replacement for `import OpenAI from 'openai'`.
 *
 * @example
 * const { OpenAI } = require('openai-oxide/compat');
 * const client = new OpenAI();
 * const r = await client.chat.completions.create({ model: 'gpt-5.4-mini', messages: [...] });
 */
class OpenAI {
  constructor(opts = {}) {
    // Official SDK reads OPENAI_API_KEY from env automatically
    this._native = new Client();
    this.chat = new Chat(this._native);
    this.responses = new Responses(this._native);
  }
}

module.exports = { OpenAI, default: OpenAI };
