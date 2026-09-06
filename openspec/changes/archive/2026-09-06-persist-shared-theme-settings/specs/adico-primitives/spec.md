## ADDED Requirements

### Requirement: Persisted app-wide settings share one primitive
A setting that is shared app-wide and SHALL survive a reload SHALL be
expressed as a module-level `GlobalSignal` driven through the crate's single
persisted-global primitive, rather than each consumer hand-rolling its own
storage read/write. The primitive SHALL support multiple independent keys,
persisting to `localStorage` on `web` and to a per-key preferences file on
`native`, and SHALL behave as a plain in-memory shared signal (no
persistence) when neither client feature is enabled. It SHALL NOT require
the consuming application to mount a provider component.

#### Scenario: A second persisted setting is added
- **WHEN** a new shared, reload-surviving setting is introduced (for example
  a second registry component's own preference)
- **THEN** it composes the existing persisted-global primitive with its own
  `GlobalSignal` and storage key, rather than a new hand-written storage
  read/write implementation

#### Scenario: A persisted setting is read on a build with neither client feature
- **WHEN** a consumer builds with neither the `web` nor the `native` feature
  enabled
- **THEN** the setting behaves as a plain in-memory shared signal, defaulted
  to its declared default, with no panic and no attempted storage access

#### Scenario: Two components read the same persisted setting simultaneously
- **WHEN** two components are mounted at the same time and both read the
  same persisted setting
- **THEN** both observe one live, shared value, and a change from either one
  is immediately observed by the other with no reload needed
