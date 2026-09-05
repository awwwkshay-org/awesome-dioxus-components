## MODIFIED Requirements

### Requirement: Removed platform coverage is recorded as a named gap
When an example fixture that was the sole evidence source for a piece of
platform coverage is removed, that removal SHALL be recorded as an explicit,
named gap in the relevant OpenSpec change's task notes rather than left
silently unrecorded or claiming a passing validation that no longer has
evidence behind it.

#### Scenario: Desktop example fixture is removed
- **WHEN** `examples/desktop` is deleted and no other fixture builds against
  a native desktop target
- **THEN** the change that removes it records, in its own tasks/design notes,
  which validation claims previously relied on `examples/desktop` and that
  they are now unverified, rather than continuing to claim passing coverage
