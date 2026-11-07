# touchstone Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- VL-1. Every proposed setup change becomes a tracked experiment with a baseline, a result, and a verdict. No change is verdict free.
- VL-2. The registry is fed by hand at first. Automation earns its way in later.
- VL-3. Touchstone judges changes, tameshi judges checks. A tameshi ruling registers here as an experiment.
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
- VL-14. A Rust library crate plus a thin touchstone
  binary. The flighttape crate at pinned revision
  ba8d4b550850179722f48b8f9c909e199efe3548 is the only
  direct dependency: the JSON reader, the value model,
  and the timestamp grammar come from the crate, never
  re-implemented. The registry format is touchstone's
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
  touchstone's card digest is built over a
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
- VL-31. Round 5 takes the registry path itself
  as its subject. Every other crate in this line
  has now had its path handling probed under the
  families the rotation measured (deleted
  working directories, tailed spellings, links
  and chains, and, in secondbill's Round 5, the
  temp placement law); this crate's registry
  argument has never had the same pass. The
  probe set: relative and absolute registry
  spellings from a deleted working directory on
  every command; tailed registry spellings
  (trailing separator, dot component, doubled
  separators, anchors); the registry spelled
  through live links, dangling links, chains,
  and loops, for reads and for appends alike;
  and any surface in this crate that hands a
  path to a library call whose own path handling
  could differ from a plain open. Each family is
  measured against the oracle before any repair
  is written, and the crate's own postures
  (named refusals, the standing crash-class
  deviation) decide what faithful means per
  cell.
- VL-32. The CodeRabbit disposition owed from
  Round 4 discharges here on evidence: the
  Round 4 pull request carries zero inline
  findings on read-back (checked 2026-11-04
  alongside the whole rotation's late shells,
  all zero), so there is nothing to fix and the
  absence is the disposition.
- VL-33. Repairs follow the measurements under
  new clauses if a probe set expands; the
  differential harness gains the proven cells
  with control runs against the pre-repair
  binary failing exactly the new cells, and the
  standing harness stays clean.
- VL-34. README records 5 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- VL-35. Amendment, from the Round 5 probes
  (written before any repair). From a deleted
  working directory, reads and appends agree on
  every command, but the view guard's resolution
  consults the working directory for relative
  spellings: the Python's resolve() raises there
  uncaught, while the port falls back to a root
  spelling and proceeds, in two cells writing
  the view where the Python crashed. A failed
  working-directory read on a relative spelling
  fails the resolution outright, and the guard
  takes the crash class (VL-13's posture),
  exactly as the Python's uncaught raise
  behaves.
- VL-36. The guard's resolution is the
  interpreter's walk, not a canonicalize with a
  lexical peel. Measured against the Python's
  resolve(): a dangling final link resolves to
  its target spelling read against the link's
  parent directory (a dangling link naming a
  missing registry still refuses as the view
  overwriting it); a link loop that is not
  cancelled by a following dotdot fails the
  resolution and takes the crash class (the
  port today writes over the loop link); nested
  splices past the kernel's open limit still
  resolve by the walk, up to this command's
  measured recursion band: a chain of 991
  nested links onto the registry resolves and
  refuses, and the 992nd fails the resolution
  and takes the crash class (the port today
  writes over the chain head at every depth
  past 40); a loop cancelled by a following
  dotdot resolves as the walk resolves it.
- VL-37. The view's temporary file is created
  under the lexically absolutized parent of
  the constructed output path: the working
  directory joined when the parent is relative,
  then lexically normalized with no link
  resolution, exactly where tempfile.mkstemp
  places it in the Python; the rename still
  targets the constructed path. Measured: when
  the output's parent pops a symlink with a
  dotdot and only the physically resolved
  parent exists, the port writes where the
  Python refuses that it cannot write the
  view.
- VL-38. open's existence probe mirrors
  pathlib's exists(): a stat failure in the
  not-found class (a missing component, a
  component that is not a directory, a loop)
  still reads as absence, and any other stat
  failure propagates as the bare OS-error
  refusal the Python's main handler prints.
  Measured as uid 65534, since the differential
  harness runs as root and cannot carry the
  cell (the probes are the pin): open with the
  registry under an unsearchable directory,
  where the port today answers with its
  cannot-append phrase. Result, check, and
  list under the same directory already agree,
  because the load speaks first.
- VL-39. The absent classes, measured and
  pinned with no repair: deleted-working-
  directory reads and appends on every command
  (plain opens never consult the working
  directory); tailed registry spellings (a
  trailing separator, a dot or doubled
  component, the anchor forms, the empty
  spelling) on reads, appends, and fresh
  opens alike; registry reads and appends
  through live links, dangling links, chains,
  and loops, including a fresh registry
  created through a dangling link and the
  Errno 40 refusals a 41-link chain names
  identically on both sides; and the judge
  fixture read, a plain read of the
  constructed path whose spellings naming the
  same file behave identically and whose
  unreadable spellings refuse with the one no
  judgments file text, while the connector
  script in JEV_CALL is handed to the
  interpreter as an argument and never opened
  by this crate.
- VL-40. Round 6 takes the view temporary's
  name as its subject. VL-37 settled where the
  temporary file is created; what it is called
  has never been measured in this crate. The
  name has two parts with an oracle law each:
  the suffix tempfile.mkstemp appends to the
  prefix it is handed, and the output's name
  the prefix carries. Both are measured
  against the Python at the view seam (list
  and show writing to an output path), at the
  name-length boundary the filesystem sets,
  before any repair is written.
- VL-41. The CodeRabbit dispositions owed
  from Round 5 and from the rename discharge
  here on evidence, in VL-32's shape: the
  Round 5 pull request (PR 7) carries zero
  inline findings on read-back (checked
  2026-11-06: zero review comments and zero
  reviews; its summary shell finalized after
  the merge as a walkthrough recording that
  the review closed with the pull request,
  and no finding was ever posted), and the
  rename pull request (PR 8) carries zero
  inline findings on read-back (the same
  check: zero review comments and zero
  reviews, and its summary states no
  actionable comments were generated), so
  there is nothing to fix and the absences
  are the dispositions.
- VL-42. The suffix law, measured: mkstemp
  appends exactly eight characters to the
  prefix (the name and a dot), so the
  Python's temporary candidate is the output
  name plus nine characters, and against this
  filesystem's 255-byte component limit the
  Python writes a view to names of up to 246
  bytes and refuses at 247. The port's
  generator draws nine bytes and renders each
  as two hexadecimal characters, a constant
  eighteen: its candidate is the name plus
  nineteen, and it refuses names of 237 to
  246 bytes the Python writes, answering that
  it cannot write the view where the Python
  wrote. The repair renders exactly eight
  hexadecimal characters from four drawn
  bytes, the fallback's sixteen-byte mix
  folded by lanes so every term of it still
  reaches the drawn bytes; the draw's sources
  and the exclusive-create retry loop stand
  unchanged.
- VL-43. The name the prefix carries is the
  output's raw name, byte for byte. The
  Python hands mkstemp the name as spelled,
  and surrogateescape round-trips its raw
  bytes; the port builds its candidate's name
  through to_string_lossy, so a raw byte in
  the name becomes the three bytes of U+FFFD
  and the candidate runs two bytes longer
  than the name it stands for: a name carrying
  one raw byte meets the port's refusal two
  lengths before even VL-42's boundary, at
  235, while the Python writes it through
  246. The repair assembles the candidate
  from raw bytes end to end (the temporary
  parent's bytes, the name's bytes, the dot,
  the suffix), the shape SB-37 settled in
  secondbill; the rename target and the
  cannot-write message keep the raw spelling
  they already keep.
- VL-44. The differential harness gains the
  boundary cells in both arms: plain names at
  the lengths around both flips (236, 237,
  246, 247) and names carrying one raw byte
  at the lengths around theirs (234, 235,
  245, 246, 247), each plain and through a popped
  parent, with control runs against the
  pre-repair binary failing exactly the new
  cells and the standing harness clean.
  README records 6 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner
  verification.
- VL-45. Two wording corrections to this
  round's own clauses, on the spec review's
  Minors. VL-41's account of PR 7's summary
  shell is exact as follows: the shell
  records the review as failed because the
  pull request closed before the review
  completed; no finding was ever posted, and
  the disposition stands on the zero-comment
  read-back, not on the shell's wording.
  VL-44's cells are not a full cross product
  of the plain and popped forms: the plain
  arm pins 236, 237, 246, and 247 (with show
  at 246 and 247), the popped arm pins 237,
  246, and 247; the raw arm pins 234, 235,
  245, 246, and 247 (with show at 246), the
  raw popped arm pins 235, 246, and 247. The
  popped arms omit plain 236, raw 234, and
  raw 245 by design: the convictions those
  lengths carry are pinned in the plain arms
  and at raw popped 246.
- VL-46. Round 6's CodeRabbit disposition,
  owed to this round's contract as late
  findings by Round 6's own record, is
  discharged here on a full read-back of
  PR 9 taken on 2026-11-07, all three list
  endpoints read to their ends. Inline
  review comments: none. Reviews: exactly
  one, a COMMENTED review submitted
  2026-11-06T10:59:20Z, before the merge
  at 11:03:06Z; its body posts no finding,
  and its one actionable item is a failed
  pre-merge check, Docstring Coverage,
  asking for docstrings on the functions
  the Round 6 diff touched, to meet an 80
  percent threshold. Issue comments: four,
  the trigger, the summary shell, one
  auto-reply, and the landing comment. The
  shell's final form is a completed
  walkthrough (created 10:56:04Z, last
  updated 10:59:22Z) stating that no
  merge-blocking issue was identified,
  with pre-merge checks at four passed and
  one failed warning, the same Docstring
  Coverage item. It does not carry the
  failed-because-closed shape of the
  sibling dispositions, because it
  completed before the merge. Round 6's
  record, that the shell stood in progress
  at the merge with zero findings, was
  stale in the sibling way: the shell had
  finalized before the merge and the
  merge-time read missed its final form.
  The auto-reply (10:59:40Z) records an
  action not completed against a
  finishing-touches interaction (already
  reviewed the last commit); it carries no
  finding. The one item is dismissed, with
  its reason stated here. The check ran
  under CodeRabbit's default profile with,
  by its own run configuration, no
  configuration file from this repository
  applied, so the threshold it enforces is
  that default, not a standard of this
  repo. The two functions it names,
  write_atomically and random_suffix in
  src/cli.rs, are private helpers, and
  each carries explanatory comments in its
  body in the crate's standing style,
  citing the clauses it implements (VL-43
  and VL-42); the check counts doc
  comments only, so what it reports is the
  comment style, not an absence of
  documentation. It identifies no wrong
  behavior. CodeRabbit stands advisory in
  this line, never a gate, and no source
  change is made for this item.
- VL-47. The README is rewritten to the
  fleet README standard. The first screen
  carries, in order: the title; the lede,
  the repo description carried verbatim
  (An experiment registry for a fleet's
  setup. Every proposed change becomes a
  tracked experiment with a hypothesis, a
  run, measurements, and a verdict, keep
  or kill.); one short paragraph stating
  what the software is, what it does, and
  how it is used; the one badge; and one
  block of real captured output. Then
  Install, Quickstart, Usage, Status, and
  License, in that order. The badge is the
  ci workflow's Actions badge, verified
  before it ships: the workflow stands
  active, its latest run on main completed
  success, and the badge image itself was
  fetched and renders ci - passing. The
  description line carries no colon and no
  semicolon, so carrying it verbatim needs
  no exception to the standard's copy bar,
  unlike the sibling case where the pinned
  line carries a colon (OR-39). Every
  capture in the README is real: the
  Install block runs verbatim (cargo
  install --path . into a scratch
  CARGO_HOME), the Quickstart runs the
  installed command end to end (open,
  result, close), and the Usage outputs
  are captured from the same binary. Every
  number in the README traces to the
  repo's own records. This subject touches
  README.md only.
- VL-48. The third home of the one line
  description does not agree. Cargo.toml's
  description reads Every setup change as
  a tracked experiment, with a verdict, a
  third variant against the repo
  description and the README lede of
  VL-47. It is named here as an open item
  outside this round's README-only bound,
  and it is not fixed in this round.
- VL-49. Regression evidence and closure.
  No source file changes in this round.
  The standing differential harness (198
  cases, 400 steps, in its standing form)
  runs against this round's tip binary and
  reads byte-clean, and the four-check
  gate (cargo test at the standing 44,
  cargo clippy --all-targets -- -D
  warnings, cargo fmt --check, and the
  codepoint scan over all tracked files)
  passes on the final tree. README records
  7 of 100 only in the closure commit
  written after this round's three
  reviews, fixes, and owner verification.
- VL-50. A repair to the README on the
  red team's Minor, owed to this round's
  closure. The Usage paragraph said the
  judgment and its standing are recorded
  in the experiment's record either way.
  That is wrong for two of the four
  consult outcomes. When the judge
  abstains, and when the judge answers
  below the confidence floor, the close
  does not land, the experiment stays
  open, and nothing is recorded, as
  VL-24a contracts and the binary does.
  The paragraph now states the outcomes
  exactly: a consulted judge's answer is
  recorded with the close when the close
  lands, and on an abstention or a
  below-floor answer the close does not
  land, the experiment stays open, and
  nothing is recorded.
- VL-51. A wording correction to VL-46,
  on the spec review's Minor, in the
  VL-45 class. VL-46 says the one
  actionable item names the two
  functions, write_atomically and
  random_suffix. The Docstring Coverage
  check names no function: neither the
  review body nor the shell's Docstring
  row names either function, and the
  item scopes itself to the functions
  touched by the diff in general. The
  two names come from the Round 6 diff
  itself and from the shell's file
  summary. VL-46's dismissal stands on
  that restated basis, its substance
  unchanged: the threshold the check
  enforces is CodeRabbit's default, the
  two helpers are private and each
  carries explanatory comments in its
  body citing the clauses it implements
  (VL-43 and VL-42), the check counts
  doc comments only so what it reports
  is the comment style and not an
  absence of documentation, it
  identifies no wrong behavior, and
  CodeRabbit stands advisory in this
  line, never a gate.
