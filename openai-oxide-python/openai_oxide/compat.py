"""
Drop-in compatibility layer matching the official `openai` Python SDK.

Usage — change one import:

    # Before (official SDK)
    from openai import AsyncOpenAI
    client = AsyncOpenAI()

    # After (openai-oxide — 3-14% faster)
    from openai_oxide.compat import AsyncOpenAI
    client = AsyncOpenAI()

    # Same code works:
    r = await client.chat.completions.create(model="gpt-5.4-mini", messages=[...])
    print(r.choices[0].message.content)
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Any, Optional

from openai_oxide import Client as _RustClient


def _ensure_strict(schema: dict) -> None:
    """Enforce OpenAI strict schema rules: additionalProperties, required."""
    if not isinstance(schema, dict):
        return
    if schema.get("type") == "object":
        schema.setdefault("additionalProperties", False)
        if "properties" in schema:
            schema["required"] = list(schema["properties"].keys())
            for prop in schema["properties"].values():
                _ensure_strict(prop)
    if "items" in schema:
        _ensure_strict(schema["items"])
    for key in ("anyOf", "oneOf", "allOf"):
        if key in schema and isinstance(schema[key], list):
            for item in schema[key]:
                _ensure_strict(item)
    for key in ("$defs", "definitions"):
        if key in schema and isinstance(schema[key], dict):
            for defn in schema[key].values():
                _ensure_strict(defn)


# ── Response objects (match official SDK attribute access) ──

@dataclass
class _Usage:
    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0
    input_tokens: int = 0
    output_tokens: int = 0


@dataclass
class _FunctionCall:
    name: str = ""
    arguments: str = ""


@dataclass
class _ToolCall:
    id: str = ""
    type: str = "function"
    function: _FunctionCall = field(default_factory=_FunctionCall)


@dataclass
class _Message:
    role: str = "assistant"
    content: Optional[str] = None
    refusal: Optional[str] = None
    tool_calls: Optional[list[_ToolCall]] = None


@dataclass
class _Choice:
    index: int = 0
    message: _Message = field(default_factory=_Message)
    finish_reason: Optional[str] = None


@dataclass
class _ChatCompletion:
    id: str = ""
    object: str = "chat.completion"
    created: int = 0
    model: str = ""
    choices: list[_Choice] = field(default_factory=list)
    usage: Optional[_Usage] = None

    @property
    def output_text(self) -> str:
        if self.choices and self.choices[0].message.content:
            return self.choices[0].message.content
        return ""


@dataclass
class _ParsedMessage(_Message):
    parsed: Any = None


@dataclass
class _ParsedChoice(_Choice):
    message: _ParsedMessage = field(default_factory=_ParsedMessage)


@dataclass
class _ParsedChatCompletion(_ChatCompletion):
    choices: list[_ParsedChoice] = field(default_factory=list)


def _build_completion(raw: dict) -> _ChatCompletion:
    """Convert raw JSON dict to _ChatCompletion object."""
    choices = []
    for c in raw.get("choices", []):
        msg = c.get("message", {})
        tool_calls = None
        if msg.get("tool_calls"):
            tool_calls = [
                _ToolCall(
                    id=tc.get("id", ""),
                    type=tc.get("type", "function"),
                    function=_FunctionCall(
                        name=tc.get("function", {}).get("name", ""),
                        arguments=tc.get("function", {}).get("arguments", ""),
                    ),
                )
                for tc in msg["tool_calls"]
            ]
        choices.append(
            _Choice(
                index=c.get("index", 0),
                message=_Message(
                    role=msg.get("role", "assistant"),
                    content=msg.get("content"),
                    refusal=msg.get("refusal"),
                    tool_calls=tool_calls,
                ),
                finish_reason=c.get("finish_reason"),
            )
        )
    usage_raw = raw.get("usage")
    usage = None
    if usage_raw:
        usage = _Usage(
            prompt_tokens=usage_raw.get("prompt_tokens", 0),
            completion_tokens=usage_raw.get("completion_tokens", 0),
            total_tokens=usage_raw.get("total_tokens", 0),
            input_tokens=usage_raw.get("input_tokens", 0),
            output_tokens=usage_raw.get("output_tokens", 0),
        )
    return _ChatCompletion(
        id=raw.get("id", ""),
        object=raw.get("object", "chat.completion"),
        created=raw.get("created", 0),
        model=raw.get("model", ""),
        choices=choices,
        usage=usage,
    )


# ── Resource classes (match client.chat.completions.create pattern) ──

class _Completions:
    def __init__(self, rust_client: _RustClient):
        self._client = rust_client

    async def create(
        self,
        *,
        model: str,
        messages: list[dict],
        **kwargs,
    ) -> _ChatCompletion:
        request = {"model": model, "messages": messages, **kwargs}
        raw_json = await self._client.create_chat_raw(json.dumps(request))
        raw = json.loads(raw_json)
        return _build_completion(raw)

    async def parse(
        self,
        *,
        model: str,
        messages: list[dict],
        response_format: type | None = None,
        **kwargs,
    ) -> _ParsedChatCompletion:
        request: dict[str, Any] = {"model": model, "messages": messages, **kwargs}

        if response_format is not None:
            # Pydantic v2 BaseModel
            schema = response_format.model_json_schema()
            _ensure_strict(schema)
            request["response_format"] = {
                "type": "json_schema",
                "json_schema": {
                    "name": response_format.__name__,
                    "schema": schema,
                    "strict": True,
                },
            }

        raw_json = await self._client.create_chat_raw(json.dumps(request))
        raw = json.loads(raw_json)
        completion = _build_completion(raw)

        # Parse content into response_format model
        parsed_choices = []
        for choice in completion.choices:
            parsed = None
            if response_format and choice.message.content:
                data = json.loads(choice.message.content)
                parsed = response_format.model_validate(data)
            parsed_choices.append(
                _ParsedChoice(
                    index=choice.index,
                    message=_ParsedMessage(
                        role=choice.message.role,
                        content=choice.message.content,
                        refusal=choice.message.refusal,
                        tool_calls=choice.message.tool_calls,
                        parsed=parsed,
                    ),
                    finish_reason=choice.finish_reason,
                )
            )

        return _ParsedChatCompletion(
            id=completion.id,
            object=completion.object,
            created=completion.created,
            model=completion.model,
            choices=parsed_choices,
            usage=completion.usage,
        )


class _Chat:
    def __init__(self, rust_client: _RustClient):
        self.completions = _Completions(rust_client)


class _Responses:
    def __init__(self, rust_client: _RustClient):
        self._client = rust_client

    async def create(self, *, model: str, input: str, **kwargs) -> Any:
        resp_json = await self._client.create(model, input, **kwargs)
        return json.loads(resp_json)

    async def parse(self, *, model: str, input: str, text_format: type, **kwargs) -> Any:
        return await self._client.create_parsed(model, input, text_format, **kwargs)


# ── Main client ──

class AsyncOpenAI:
    """Drop-in async replacement for `openai.AsyncOpenAI`.

    ```python
    from openai_oxide.compat import AsyncOpenAI

    client = AsyncOpenAI()  # or AsyncOpenAI(api_key="sk-...")
    r = await client.chat.completions.create(model="gpt-5.4-mini", messages=[...])
    print(r.choices[0].message.content)
    ```
    """

    def __init__(self, *, api_key: str | None = None, base_url: str | None = None):
        self._rust = _RustClient(api_key=api_key, base_url=base_url)
        self.chat = _Chat(self._rust)
        self.responses = _Responses(self._rust)
