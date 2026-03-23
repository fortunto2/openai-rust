// Optional Zod integration for structured outputs.
// Requires: npm install zod-to-json-schema
//
// Usage:
//   const { zodParse, zodResponseParse } = require('openai-oxide/zod');
//   const { z } = require('zod');
//
//   const Answer = z.object({ text: z.string(), confidence: z.number() });
//   const { parsed } = await zodParse(client, request, Answer);

let zodToJsonSchema;
try {
  zodToJsonSchema = require('zod-to-json-schema').zodToJsonSchema;
} catch { /* optional dep */ }

function ensureZod() {
  if (!zodToJsonSchema) {
    throw new Error('Install zod-to-json-schema: npm install zod-to-json-schema');
  }
}

async function zodParse(client, request, schema) {
  ensureZod();
  const jsonSchema = zodToJsonSchema(schema);
  const name = schema.description || 'Response';
  const result = await client.createChatParsed(request, name, jsonSchema);
  result.parsed = schema.parse(result.parsed);
  return result;
}

async function zodResponseParse(client, request, schema) {
  ensureZod();
  const jsonSchema = zodToJsonSchema(schema);
  const name = schema.description || 'Response';
  const result = await client.createResponseParsed(request, name, jsonSchema);
  result.parsed = schema.parse(result.parsed);
  return result;
}

module.exports = { zodParse, zodResponseParse };
