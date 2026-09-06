## ADDED Requirements

### Requirement: Clipboard copy is a target-gated shared primitive
`adico-primitives` SHALL expose a hook that copies a given text value to the
system clipboard, resolving to a status (idle, copied, or failed) rather than
a bare boolean, so a consumer can render a confirmation without maintaining
its own timer. On the `web` target it SHALL use the browser clipboard API. On
every other target it SHALL resolve to a failed status rather than silently
appearing to succeed, since no native clipboard integration exists in this
crate. Browser-interop details (the actual clipboard API call) SHALL stay
inside this primitive, never called directly from registry UI source.

#### Scenario: A web consumer copies text
- **WHEN** a `web`-target consumer calls the clipboard hook's copy function
  with a text value
- **THEN** the value is written to the system clipboard and the hook's status
  resolves to "copied"

#### Scenario: Clipboard access is denied or unsupported
- **WHEN** the browser denies clipboard permission, or the running target has
  no clipboard integration
- **THEN** the hook's status resolves to "failed", not "copied" and not a
  silent no-op

#### Scenario: Status is transient
- **WHEN** a copy attempt resolves to "copied" or "failed"
- **THEN** the status returns to "idle" after a short, fixed delay without
  the consumer needing to manage that timing itself
