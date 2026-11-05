# verdictloop

The experiment registry that closes the retro loop, where every proposed setup change from any tool in this family becomes a tracked experiment with a baseline, a result, and a verdict: keep, kill, or rerun.

## What this is

Setup changes ship on faith, and findings from every other instrument evaporate because nothing keeps the lab notebook. Verdictloop keeps it. The registry is an append-only file of events, opened, result, and closed, fed by hand at first; state is folded from the file on every read and nothing is stored beside it. Opening records the source tool, the proposal, the experiment id as the source derived it (retroforge's exp- ids are the first case), a baseline statement, the baseline measures, and the measures the experiment will be judged on, with their directions and margins. Results accumulate; the latest is current. Closing rests on the baseline and the current result, named by their positions in the file, and records the closer's verdict beside a suggested verdict computed by a stated rule. The suggestion never replaces the verdict, disagreements stand as part of the record, and the recorded suggestion is re-derived at read time, so a forged receipt names itself as a defect. Tameshi judges checks, verdictloop judges changes, and a tameshi ruling registers here as an experiment.

## Status

Rounds 1 and 2 are landed. Round 2 ported the registry to Rust,
byte for byte against the Python it replaced; the Python is
retired and the crate is the implementation now. Setup landed
earlier in October 2026.

## Programme status

- Closed perfection rounds, 5 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, VL-1 to VL-19 as amended, met for the Round 2 scope.
- Local gate, python3 scripts/codepoint_scan.py, cargo
  fmt --check, cargo clippy --all-targets -- -D warnings,
  and cargo test. CI runs the same commands.

## Licence

MIT. See LICENSE.
