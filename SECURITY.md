# Security Policy

## Supported versions

tableflow is pre-release (`0.0.x`); only the latest published version receives
security fixes.

## Reporting a vulnerability

**Please do not open a public issue for security problems.**

Report privately through GitHub: on the repository's **Security** tab, choose
**Report a vulnerability** (GitHub Private Vulnerability Reporting). If that is
unavailable, contact the maintainer
[@horacioskrp](https://github.com/horacioskrp) and ask for a private channel.

Please include:

- affected version(s) and platform,
- a minimal form (content) + submissions that reproduce the issue,
- the impact you observed (e.g. panic / crash, resource exhaustion, or an export
  that silently misrepresents the collected data).

We aim to acknowledge a report within a few days, agree on a disclosure
timeline, and credit reporters who wish to be named once a fix is released.

## Scope & threat model

tableflow is a data-export library: it reads untrusted form definitions and
submissions and produces tabular files (CSV / XLSX / GeoJSON). Security-relevant
issues include memory-safety problems (the workspace sets
`unsafe_code = "forbid"`, so these should not occur), panics or unbounded
resource use on crafted input, spreadsheet-formula injection in generated
cells, and output that silently diverges from the submitted data. The crate
performs no network access of its own.
