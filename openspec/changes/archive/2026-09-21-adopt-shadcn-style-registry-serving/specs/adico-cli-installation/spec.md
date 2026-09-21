## ADDED Requirements

### Requirement: Official registry resolution has a network-independent fallback
The CLI SHALL resolve the official `@adico` registry over the network by
default. When network resolution is unavailable or fails, the CLI SHALL fall
back to a committed, self-contained snapshot of the official registry so that
`adico init`, `adico add`, `adico list`, and `adico view` continue to work
against the official registry without network access.

#### Scenario: Official registry resolves over the network
- **WHEN** a consumer runs `adico add @adico/button` with network access
  available
- **THEN** the CLI resolves and installs Button using the network-resolved
  official registry

#### Scenario: Network is unavailable
- **WHEN** a consumer runs `adico add @adico/button` with no network access
  available
- **THEN** the CLI resolves and installs Button using the committed offline
  snapshot of the official registry, without failing the request

#### Scenario: Offline resolution reports its source
- **WHEN** the CLI falls back to the committed offline snapshot of the
  official registry
- **THEN** the CLI's reviewable plan or result identifies that the official
  registry was resolved from the offline snapshot rather than the network
