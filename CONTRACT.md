# verdictloop Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- VL-1. Every proposed setup change becomes a tracked experiment with a baseline, a result, and a verdict. No change is verdict free.
- VL-2. The registry is fed by hand at first. Automation earns its way in later.
- VL-3. Verdictloop judges changes, keeporkill judges checks. A keeporkill ruling registers here as an experiment.
- VL-4. Every fixture committed to this repo is synthetic.

## Round 1. The registry (VL-5 to VL-12)

Contract preparation only in this pass. The build opens
when this repo's turn in the line's build order arrives;
verdictloop builds last in the line, and the registry
stub retroforge already emits is its first feeder.

- VL-5. The registry is an append-only JSON lines file
  of events: opened, result, and closed. Each event
  carries the experiment id, a timestamp the caller
  supplies (the registry invents no times), and its
  payload. Registry state is derived by folding the
  events; no state is stored beside the file. A line
  that fails the event shape is a named defect at read
  time, with its line number. Test,
  tests/test_registry.py.
- VL-6. Opening an experiment records the source tool,
  the proposal id the source gave it, the experiment id
  as the source derived it (retroforge's exp- ids are
  the first case), a baseline statement naming what the
  baseline measures, the baseline measures as a JSON
  object, and the measure names the experiment will be
  judged on. Opening an id that already exists is
  refused by name. Test, tests/test_registry.py.
- VL-7. A result records follow-up measures in the same
  shape as the baseline. A result for an unopened or a
  closed experiment is refused by name. Results
  accumulate, each with its own timestamp; the latest is
  the current result, and earlier ones stay in the
  history. Test, tests/test_registry.py.
- VL-8. Closing records a verdict from a fixed set
  (keep, kill, rerun), the closer's rationale, and the
  baseline and current result it rests on, named by
  their event positions. Closing an unopened or already
  closed experiment is refused by name. A rerun verdict
  may name the successor experiment, which is opened as
  its own experiment when it exists. Test,
  tests/test_registry.py.
- VL-9. Every close also records a suggested verdict,
  computed by a stated deterministic rule over the
  baseline and current measures: keep when no judged
  measure regressed past the margin the opening stated,
  kill when two or more regressed, rerun otherwise. The
  suggestion is recorded beside the verdict, never in
  place of it; where the two disagree, both stand, and
  the disagreement is itself part of the record. The
  decision-model round may later argue the disagreement;
  Round 1 calls no model. Test, tests/test_registry.py.
- VL-10. Views: list shows every experiment with its
  state (open, resulted, or closed with its verdict),
  and show returns one experiment's full event history
  with the baseline and results as its receipts. Both
  views are deterministic folds of the same file.
  Test, tests/test_registry.py.
- VL-11. A check command reads the whole registry and
  names every defect it finds: bad lines, events for
  unopened experiments, duplicate openings, closes that
  name missing results. Nothing is silently repaired.
  Test, tests/test_registry.py.
- VL-12. Writes follow the family's append discipline:
  one line, written and flushed in a single append, with
  a crash mid-append detectable by check as a defective
  final line. Rendered views go to stdout or a named
  path, atomically. Every failure is a named error with
  exit 1. Test, tests/test_registry.py.

## Round 1 amendments, the pinned shapes (recorded before code)

- VL-5a. An event is a JSON object carrying type
  (opened, result, or closed), experiment (the id
  string), at (a caller-supplied ISO timestamp), and
  its payload. Positions are 1-based line numbers in
  the file, the same numbers check reports. Measures
  are finite numbers; a line holding NaN or an
  infinity is a defect. Test, tests/test_registry.py.
- VL-6a. The opened payload: source, proposal,
  baseline (a statement naming what the baseline
  measures), baseline_measures (an object of measure
  name to number), judged (a list of objects with
  name, direction lower or higher, and an optional
  margin_pct), and margin_pct (the opening's default
  margin, 10 when unstated). Every judged name must
  be a key of baseline_measures or the opening is
  refused by name. Test, tests/test_registry.py.
- VL-8a. The closed payload: verdict, rationale,
  rests_on naming the baseline by the opened event's
  position and the result by its position, and, for
  a rerun verdict, optionally successor naming the
  successor experiment's id. Closing rests on the
  current (latest) result; closing with no result is
  refused by name. The successor is opened as its
  own experiment through the open command, never
  implicitly by the close. Test,
  tests/test_registry.py.
- VL-9a. The suggestion is computed at close time
  over the judged measures present in both the
  baseline and the current result, and recorded in
  the closed event with the regressed measure names.
  A measure regresses when it moves against its
  direction by more than its margin percent of the
  baseline; a zero baseline regresses for a lower
  measure on any positive result, and never for a
  higher measure. Judged measures missing from the
  result are named in the closed event as unmeasured
  and count neither way. Test, tests/test_registry.py.
- VL-11a. Reads for the views and for every write
  refuse a defective registry, naming the defects;
  check reports them and exits 1 when any exist.
  Defects: a line that is not JSON or not an event
  object, duplicate JSON keys, unknown types,
  missing or mistyped fields, events for unopened
  experiments, duplicate openings, results after a
  close, second closes, and closes whose rests_on
  names a position that is not that experiment's
  opened or result event. Test,
  tests/test_registry.py.
