# verdictloop

The experiment registry that closes the retro loop, where every proposed setup change from any tool in this family becomes a tracked experiment with a baseline, a result, and a public keep or kill verdict.

## What this is

Setup changes ship on faith, and nothing replays recorded history to prove a new rule or hook would have caught the failures without breaking the passes, so findings from every other instrument evaporate. Verdictloop keeps the lab notebook for all of them. A proposal enters with its baseline, measurements attach as they land, and the experiment closes with a verdict and the fleet metrics that decided it. The registry itself becomes the public proof that the whole retrospective program works. The boundary with its sibling is one sentence. Keeporkill judges checks, verdictloop judges changes.

## Status

Seeded in October 2026. Local setup is in progress on the setup
branch. The remote seeding pass is waiting on a token access fix, so
work is banked locally until pushes clear. This repo is in phase 3
of the Retrospective Line and its build waits on the phase gate, the
main chat confirmation that the Gate Truth evidence pack schema has
merged and passed acceptance. Design work may proceed, build does
not start before the gate.

## Programme status

- Closed perfection rounds, 0 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, stub in place, not yet met.
- Local gate, python3 scripts/codepoint_scan.py and, once tests
  exist, python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.
