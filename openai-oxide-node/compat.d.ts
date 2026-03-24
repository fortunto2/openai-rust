/**
 * Drop-in compatibility layer matching the official `openai` npm package.
 */

export interface ChatCompletionMessage {
  role: string;
  content: string | null;
  refusal?: string | null;
  tool_calls?: Array<{
    id: string;
    type: string;
    function: { name: string; arguments: string };
  }>;
}

export interface ChatCompletionChoice {
  index: number;
  message: ChatCompletionMessage;
  finish_reason: string | null;
}

export interface ChatCompletion {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: ChatCompletionChoice[];
  usage?: { prompt_tokens: number; completion_tokens: number; total_tokens: number };
}

export interface ChatCompletionChunk {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: Array<{
    index: number;
    delta: { role?: string; content?: string; tool_calls?: any[] };
    finish_reason: string | null;
  }>;
}

export interface Completions {
  create(params: Record<string, any> & { stream?: false }): Promise<ChatCompletion>;
  create(params: Record<string, any> & { stream: true }): Promise<AsyncIterable<ChatCompletionChunk>>;
  parse(params: Record<string, any>): Promise<{ completion: ChatCompletion; parsed: any }>;
}

export interface Chat {
  completions: Completions;
}

export interface ResponseObject {
  id: string;
  object: string;
  model: string;
  output: any[];
  status?: string;
  usage?: Record<string, any>;
}

export interface Responses {
  create(params: Record<string, any>): Promise<ResponseObject>;
  parse(params: Record<string, any>): Promise<{ response: ResponseObject; parsed: any }>;
}

export declare class OpenAI {
  constructor(opts?: { apiKey?: string; baseURL?: string });
  chat: Chat;
  responses: Responses;
}

export default OpenAI;
