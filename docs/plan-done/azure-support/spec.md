# Specification: Azure OpenAI Support

**Track ID:** azure-support
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Add Azure OpenAI support to openai-oxide, matching the Python SDK's `AzureOpenAI` client. Azure users need different URL construction (`{endpoint}/openai/deployments/{deployment}`), an `api-version` query parameter on every request, and Azure AD token authentication as an alternative to API keys.

This is the #1 requested feature for OpenAI client libraries and the highest-priority item remaining in Priority 2 (Ergonomics). The implementation follows the Python SDK pattern: `AzureOpenAI` as a configured variant of the base client, not a separate client type. All existing resource methods (chat, embeddings, etc.) work unchanged.

## Acceptance Criteria

- [x] `AzureConfig` builder struct with: `azure_endpoint`, `azure_deployment`, `api_version`, `api_key`, `azure_ad_token`
- [x] `OpenAI::azure(config)` constructor that creates a client with Azure-specific URL and auth
- [x] `api-version` query parameter automatically appended to every request
- [x] URL constructed as `{endpoint}/openai/deployments/{deployment}` when deployment is set
- [x] URL constructed as `{endpoint}/openai` when no deployment is set
- [x] Azure AD token auth via `Authorization: Bearer {token}` header (alternative to API key)
- [x] Mutual exclusivity validation: api_key OR azure_ad_token, not both
- [x] `AzureConfig::from_env()` reads: `AZURE_OPENAI_API_KEY`, `AZURE_OPENAI_ENDPOINT`, `OPENAI_API_VERSION`, `AZURE_OPENAI_AD_TOKEN`
- [x] All existing resources (chat, embeddings, models, etc.) work through Azure client
- [x] At least 6 mockito tests covering Azure-specific behavior
- [x] Doc examples on `AzureConfig` and `OpenAI::azure()`

## Dependencies

- None (builds on existing `ClientConfig` and `RequestOptions`)

## Out of Scope

- `dyn Config` trait / generic `Client<C>` — separate track
- Dynamic deployment routing (model-based URL rewriting without explicit deployment) — follow-up
- Azure AD token _provider_ (callable that refreshes tokens) — follow-up
- Azure-specific error handling / retry headers — follow-up
- OpenRouter / Ollama / vLLM compatibility — separate roadmap item

## Technical Notes

- **Python SDK pattern**: `AzureOpenAI` subclasses `OpenAI`, overrides `_build_request()` and `_prepare_url()`. We achieve the same via a constructor that configures `ClientConfig` appropriately.
- **URL construction**: Azure uses `{endpoint}/openai/deployments/{deployment}/chat/completions?api-version=2024-02-01`. The base_url includes everything up to the deployment, and the existing relative path concatenation handles the rest.
- **Auth**: Azure supports API key via `api-key` header (NOT `Authorization: Bearer`). Azure AD uses `Authorization: Bearer {ad_token}`. The Python SDK uses a sentinel API key when AD auth is used.
- **api-version**: Passed as query param, not header. We use `default_query` in `ClientConfig` to add it to every request.
- **No generic refactor needed**: `AzureConfig.build()` returns a normal `OpenAI` client. Resources don't need to change at all.
