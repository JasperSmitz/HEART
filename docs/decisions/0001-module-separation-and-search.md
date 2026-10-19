# 0001 — Module separation and initial web search

**Status:** accepted, 2026-10-19. Source organization and initial provider only; implementation and the full stack are not approved.

## Context and options

HEART's capabilities should remain separate from its core. The user expects light personal search usage, accepts Tavily's free allowance initially, and wants a future path to fully self-hosted search. Options discussed included hosted search APIs and operating a self-hosted metasearch service. Search implementation could be embedded in the core or maintained as a separate capability.

## Decision and user involvement

The product owner selected Tavily for initial web lookup and requested separate capability code under `modules/*`, with web search under `modules/websearch/`. The architect accepts that boundary. The product owner further confirmed that modules must provide an established response structure while keeping their execution opaque to the core. Core code should consume declared capabilities and shared results without depending on search-domain or Tavily API details. Exact fields and input contracts remain to be specified; MCP is an analogy for the boundary, not an adopted protocol.

## Consequences and open details

- Initial work can focus on HEART without operating a search service.
- Search implementation remains replaceable as self-hosting becomes worthwhile.
- Module interfaces, package boundaries, and the later self-hosted provider remain open. Source separation alone does not imply dynamically loaded plugins or process isolation.
- No application directories or code have been created. A scoped implementation plan is still required.

The [provider pricing research](https://docs.tavily.com/documentation/api-credits) was checked on 2026-10-19; recheck allowances and terms before integration.
