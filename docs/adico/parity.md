# Component parity data

`parity.json` is the checked-in source of truth for first-party shadcn parity.
M1 adds the upstream catalog snapshot; M3 and later add one entry for each
tracked component. Company-curated registry items can carry quality and
provenance metadata, but they do not count toward official shadcn parity unless
they are explicitly mapped to an upstream catalog item.

Each applicable dimension requires evidence before it can pass. A `complete`
component must pass every dimension; an inapplicable dimension must state why.
Normal CI validates the checked-in manifest and does not request the upstream
catalog over the network. The explicit M1/M10 refresh commands are responsible
for updating the catalog snapshot.
