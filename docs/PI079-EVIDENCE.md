# PI079: independent specimens and threshold sensitivity

Issue [#180](https://github.com/UnityInFlow/injection-scanner/issues/180),
measured 2026-10-22 against the pattern on upstream `edec71c`.

The committed excerpts in `tests/fixtures/pi079-evidence.json` supply one
independent hostile command and two independent benign documentation examples.
They were selected by searching for literal key-installation commands, not
sampled randomly. None was written for this scanner. HTML entities were decoded;
the SANS excerpt selects the contiguous echo-and-append command from a longer
command chain. No key was generated, expanded from an ellipsis, or replaced.
The tests treat these commands as text and never execute them.

| Specimen and source | Classification | Shipped PI079 |
|---|---|---|
| [SANS ISC, Alex Rodriguez's DShield honeypot diary, 2023-11-27](https://isc.sans.edu/diary/30428), captured RSA key append | Hostile command | Match |
| [GeckoAdmin, SSH Key Authentication on Linux: Setup From Any Client](https://geckoadmin.com/ssh-key-authentication-setup/), Windows Ed25519 key installation | Benign documentation with a complete 68-character blob | Match (known false positive) |
| [Extract's SFTP connector documentation](https://docs.extract.to/destinations/sftp), manual key installation with an elided blob | Benign documentation | No match |

These are command excerpts, not independently observed prompt injections. The
SANS article itself is defensive documentation, not a hostile document. Its
captured command is evidence for the command shape only. This does not add an
independent prompt-injection true positive or change the existing recall pins.
The benign overlap is recorded outside the zero-finding clean corpus because
the identical command can be authorized or unauthorized. No existing clean
specimen or suppression was changed to accommodate it.

## Measurement

Run `cargo test --locked --test pi079_evidence_test`. The sensitivity test changes
only PI079's minimum in memory, and uses the production `Scanner`, including its
normalization and decoding passes. It does not alter the shipped regex.

The quantifier counts characters **after** the four literal `AAAA` characters;
the current minimum is therefore 44 base64 characters total, excluding padding.

| Minimum after prefix | Hostile command matched (of 1) | Benign documents matched (of 2) |
|---|---|---|
| 8 | 1 | 2 |
| 40 (shipped) | 1 | 1 |
| 64 | 1 | 1 |
| 65 | 1 | 0 |
| 128 | 1 | 0 |
| 512 | 0 | 0 |

The apparent improvement at 65 comes from excluding the complete Ed25519 key
(64 characters after the prefix). That excludes hostile keys of the same size
too. This small, deliberately selected sample cannot establish an optimal
cutoff, population false-positive rate, or recall rate. The large frozen
GATE-03 sweep remains a separate measurement with zero key-blob lines; these
three excerpts do not retroactively change that result.

## Decision

Retain the minimum and MEDIUM grade. The threshold remains a heuristic for short
elisions, now with measured sensitivity rather than an unsupported claim of
calibration. Length alone cannot separate legitimate key installation from
unauthorized persistence. The independent benign match supports keeping the
grade at MEDIUM under PATTERNS.md rule 3. The remediation now asks the reviewer
to verify explicit user authorization, acknowledging legitimate installations.

The existing regex is not an SSH-key parser: it neither validates decoded key
structure nor guarantees that a long elided blob is rejected. The boundary
test uses synthetic 43/44/45-character blobs only to pin those mechanics, not
as evidence of valid keys. Broader sampling across key types and independent
prompt-injection documents is still needed before claiming calibration.
