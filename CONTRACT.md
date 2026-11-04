# verdictloop Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- VL-1. Every proposed setup change becomes a tracked experiment with a baseline, a result, and a verdict. No change is verdict free.
- VL-2. The registry is fed by hand at first. Automation earns its way in later.
- VL-3. Verdictloop judges changes, tameshi judges checks. A tameshi ruling registers here as an experiment.
- VL-4. Every fixture committed to this repo is synthetic.

## Round 1. The registry (VL-5 to VL-12)

Built in Round 1, last in the line; the registry stub
retroforge already emits is its first feeder. The
round also added the amendments below (VL-5a to
VL-11a before code, VL-5c, VL-6b, VL-8b, and VL-9b
after the reviews, before the repair code).

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

## Round 1 amendments, after the reviews (recorded before the repair code)

- VL-5c. The at timestamp carries a date and a time;
  a bare date is not a timestamp. Test,
  tests/test_registry.py.
- VL-6b. Judged names are unique within an opening,
  and every margin, the opening's and each entry's,
  is non-negative. Violations are defects at read
  and refusals at write. Test, tests/test_registry.py.
- VL-8b. The result a close rests on must be the
  current one at the close's position: no later
  result of the same experiment may stand before
  the close. Verified at read as a defect, and the
  write path rests on the latest by construction.
  A successor is recorded only on a rerun verdict.
  Test, tests/test_registry.py.
- VL-9b. The recorded suggestion is re-derived at
  read from the baseline and the rested result, the
  way onrecord re-derives claim ids: a closed event
  whose suggested, regressed, or unmeasured values
  do not match the recomputation is a defect naming
  the line. The record's receipts verify themselves.
  Test, tests/test_registry.py.

## Round 2. The Rust port (VL-13 to VL-19)

Written ahead of the code, in the shape the flighttape
port set with FT-49 to FT-56 and secondtake's port
followed with ST-20 to ST-26. The Python of Round 1
is the behavioral spec.

- VL-13. The port reproduces the Python's behavior
  exactly: every command, refusal, defect phrase,
  rendered byte, and exit code. Deviations are named
  in the round dossier, never silent. Test, the
  differential harness of VL-18.
- VL-14. A Rust library crate plus a thin verdictloop
  binary. The flighttape crate at pinned revision
  ba8d4b550850179722f48b8f9c909e199efe3548 is the only
  direct dependency: the JSON reader, the value model,
  and the timestamp grammar come from the crate, never
  re-implemented. The registry format is verdictloop's
  own; no tape format, canonical form, or digest is
  consumed. Test, tests/test_registry.rs.
- VL-15. Where the registry reading differs from the
  crate reader, the Python governs: duplicate keys in
  a registry line are refused, the NaN and Infinity
  constants are refused, and a line past the Python's
  parse depth is refused, each surfacing as that
  line's not-valid-JSON defect. The port closes the
  differences in its loader and names them in the
  dossier. Test, tests/test_registry.rs.
- VL-16. Appended lines and views reproduce Python's
  json.dumps byte for byte: insertion key order, the
  default separators on lines, the indent of two on
  views, ensure_ascii=False, and exact float
  rendering. Test, tests/test_registry.rs.
- VL-17. The Python tests port to cargo tests file
  for file, and the Python package, its tests, and
  pyproject leave the tree only in the landing state.
  Cargo tests are the suite. Test, cargo test.
- VL-18. A differential harness over the whole
  command surface byte-compares stdout, stderr, exit
  codes, registry bytes, and view files against the
  Python: the happy path, the defect catalogue, every
  refusal, the write guard, and an argv battery
  across both argparse levels. Test, the harness run
  recorded in the dossier.
- VL-19. The round dossier carries the port report:
  port or full rewrite with reasons, the Rust versus
  Python line share on the landing tree, the bindings
  determination, and the non-Rust remainder register
  with a justification per item.
- VL-20. Round 3 is the decision-model round
  VL-9 foresaw. Judgment enters only where the
  recorded verdict and the deterministic
  suggestion disagree at close time; that
  disagreement is the contested band. Where
  they agree, the close records judgment status
  not_consulted with the reason that verdict
  and suggestion agree, and no model is called.
- VL-21. A contested close with no judge
  configured closes as before, its judgment
  section recording not_consulted with the
  reason that no judge is configured. The judge
  is configured the way secondtake configures
  its judge (the same transport shape and
  configuration surface), re-implemented in
  this crate; no sibling crate is consumed.
- VL-22. A contested close with a judge builds
  a decision card from the computed state (the
  opening's margins, the baseline and current
  measures, which judged measures regressed,
  the recorded verdict, the suggestion), digests
  it under canonical form 1, and asks the model
  for a choice among keep, kill, rerun, and
  abstain, with scores for evidence strength
  and regression risk and a calibrated
  confidence, exactly the roadmap's card.
- VL-23. The model's answer is recorded in the
  close event's judgment section beside the
  measures that produced it, with the model
  name and the card digest. The recorded
  verdict stands as the author's act; where
  the model's proposal differs, both stand,
  as VL-9 already provides for disagreements.
- VL-24. Where the model abstains, or answers
  under the confidence floor of 0.6, the close
  does not land: no event is appended, the
  experiment stays open, and the command
  refuses naming the abstention. Where the
  judge cannot answer (transport failure or an
  unparseable answer), the close lands with
  judgment status unavailable and is never
  marked judged. Deterministic refusals (no
  baseline, an incomplete window, a malformed
  close) precede any call, as they precede the
  close itself.
- VL-25. Tests stay offline. A bounded
  recording window captures synthetic contested
  cards and the model's answers as fixtures in
  the repo, each carrying the model name and
  the exact call shape; the suite replays them
  through a fixture transport. Live calls stay
  inside this repo's harmonized allowance (at
  most 10 verification calls, at most 200 in
  the recording window, the 5 dollar monthly
  hard stop), and the round's gate report
  prints the recorded usage.
- VL-26. The views carry the judgment: list and
  show render a closed experiment's judgment
  section where one exists, and check
  re-derives the recorded suggestion and the
  judgment's internal consistency (a judged
  close names its card digest and model; an
  abstained close does not exist, by VL-24).
- VL-27. The round's verification is the full
  local gate plus the differential harness at
  zero failures over the whole surface,
  judgment included, with the Python oracle
  for this round being Round 2's behavior on
  the uncontested surface and the contract on
  the judgment surface (the oracle predates
  the feature; the harness marks the judgment
  cells accordingly rather than dropping the
  uncontested ones).
- VL-22a. The card's full contents, enumerated
  (addendum from the Round 3 spec review, whose
  parenthetical in VL-22 named the load-bearing
  elements but not the frame): the header line
  naming the experiment and its proposal; one
  line per judged measure carrying its name,
  baseline, current, direction, effective
  margin, and outcome word; a line naming any
  judged measure the rested result did not
  measure; the regressed-measures line; and the
  verdict and suggestion line. Every element is
  deterministic from the opening and the rested
  result, and check rebuilds the card
  byte-for-byte, as it did before this addendum
  named the frame.
- VL-24a. Amendment, from the Round 3 red team:
  check holds every judgment section to its
  status's exact key set, as the fold already
  holds not_consulted to exact-object equality.
  A judged or unavailable section carrying a
  field the writer never emits is a defect.
- VL-28. Round 4 re-pins flighttape at 74a83298,
  the Round 13 merge, in every table that names
  it. Nothing else in the crate's behavior
  changes.
- VL-29. The re-pin is hygiene, not repair:
  verdictloop's card digest is built over a
  single String value, no Float can enter it,
  and Round 3's sweep digested 6,072 strings
  identically under the old pin and the Round 12
  pin. This round re-proves invariance across
  the full jump (the old pin to 74a83298) before
  landing: the recorded fixture cards digest to
  their recorded values under the new pin, the
  differential harness runs clean unmodified as
  the control, and a fresh digest sweep over
  the fixture card corpus plus generated strings
  agrees under both pins. Any digest that moves
  stops the round.
- VL-30. README records 4 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- VL-29a. Correction to VL-29, written before
  any implementation: the crate also consumes
  flighttape::timeparse::parse_timestamp
  (registry.rs, the timestamp validity check)
  and flighttape::identity::tape_digest over
  registry values, so the jump crosses both
  grammar rounds and the Round 12 rendering
  repairs. The round therefore proves three
  things by probes: the recorded fixture cards
  digest to their recorded values under the new
  pin (the card case, strings only); timestamp
  validity through the registry's surface agrees
  with the Python original on representative
  shapes from both deltas (FT-60 and FT-60a,
  FT-63 to FT-66, FT-68); and any registry value
  carrying floats that reaches tape_digest
  digests identically to the Python original
  under the new pin (the 2^-24 shape included),
  with the old pin's reading recorded either
  way. The unmodified harness runs as the
  control. Any recorded fixture digest that
  moves stops the round.
