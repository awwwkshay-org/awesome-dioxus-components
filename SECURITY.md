# Security policy

## Reporting a vulnerability

Please report security vulnerabilities privately using GitHub's
[Security Advisories](https://github.com/awwwkshay-org/awesome-dioxus-components/security/advisories/new)
for this repository ("Report a vulnerability" under the Security tab) rather
than opening a public issue. This applies to:

- `adico-cli`, `adico-primitives`, `adico-registry-core` and any other
  published crate in this workspace
- the `adico` registry format, resolution, or installation planning
- the hosted docs/playground/registry site

Include the affected version, a description of the issue, and reproduction
steps if possible. We'll acknowledge reports within a few business days.

## Scope

Components under `registry/` are source you install and then own — once
`adico add` copies a file into your project, its security is your project's
responsibility, same as any other code you write or vendor. Report issues in
the *authored* registry source (a real vulnerability in a shipped component)
here; issues in code you've since modified are yours to fix.

## Supported versions

This project is pre-1.0. Security fixes land on the latest published `0.x`
release; there is no long-term-support branch yet.
