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
- VL-52. CodeRabbit step 0 repairs on
  PR 11. Four findings against the new
  README, each verified against the
  binary or the source before the
  sentence moved. First, comment
  4501918687. The check sentence said
  check recomputes every experiment
  id. It does not. The registry
  validates each event's id as a
  non-empty string (event_defects in
  registry.rs) and the fold groups
  events by the id string, so a result
  or close naming an id no opening
  named is a defect, and what check
  recomputes is the recorded
  suggestion and a judged close's card
  digest. Demonstrated on the
  quickstart record with the shipped
  binary. The id swapped consistently
  to a second non-empty id still reads
  sound, and one event renamed alone
  fails on the unopened-experiment
  defect. The sentence now states the
  validation semantics. Second, third,
  and fourth, all under comment
  4501918694. The successor sentence
  named the flag and no restriction,
  and the band paragraph implied every
  disagreement consults a judge and a
  consulted judge's answer is what the
  close records. The paragraph now
  states that a successor is recorded
  only on a rerun verdict and a keep
  or kill close naming one is refused
  with the rule named, shown against
  the binary with the registry left
  unappended, and that naming a
  successor does not open it, the
  successor needing its own opening.
  It states that a disagreement with
  no judge named lands the close with
  the judgment recorded as not
  consulted, shown with the recorded
  reason that no judge is configured.
  And it qualifies the answer sentence
  with the unavailable outcome. A
  judge whose call fails or whose
  answer is malformed leaves the
  close landed with the judgment
  recorded as unavailable in place of
  an answer, shown with a judgments
  file holding no recorded card, the
  close landing with the card and its
  digest recorded and the status
  unavailable. No source file changes.
  README and this clause only.

## Round 8

- VL-53. Round 8 opens with the full
  read-back of pull request 11 (Round
  7), taken on 2026-11-08 through the
  API, every review comment thread,
  every review body to its end, and
  every issue comment, the follow-up
  shells and the bot acknowledgment
  replies included. Inline review
  comments, six: two CodeRabbit
  finding comments carrying the four
  findings (4501918687, the experiment
  id recomputation claim, and
  4501918694, carrying three, the
  successor restriction, the no-judge
  outcome, and the unavailable
  outcome), the owner's two fix
  replies (4501948633 and 4501948797),
  and CodeRabbit's two acknowledgment
  replies (4501950917 and 4501951533,
  each recording its thread resolved).
  Reviews, five: one CodeRabbit
  COMMENTED review (5812467162,
  submitted 2026-11-07T13:29:41Z,
  Actionable comments posted: 4, its
  run configuration stating that no
  configuration file from this
  repository was applied), the owner's
  two empty COMMENTED reviews riding
  the fix replies, and CodeRabbit's
  two empty COMMENTED reviews riding
  the acknowledgments. Issue comments,
  four: the owner's trigger
  (6707524253), the summary shell
  (6707525643, created
  2026-11-07T13:25:53Z and last updated
  2026-11-07T13:42:01Z), the action
  notice (6707570818, Already reviewed
  the last commit, carrying no
  finding), and the owner's landing
  comment (6707768518, after the merge
  at 2026-11-07T13:49:27Z). The shell
  stands in its final form as the
  rate-limit banner (Review limit
  reached, the next included review
  available in 43 minutes, its second
  run over the range from 4f385428 to
  84be44f3) over a complete
  walkthrough of the first head, with
  Merge Risk Low up to 4f385428 and
  its pre-merge checks at four passed
  (Description, Title, Linked Issues,
  and Out of Scope Changes), no failed
  check among them. The four findings
  stand disposed in VL-52, repaired
  before the merge, replied to on
  their threads, and acknowledged by
  the bot, and the VL-52 head drew no
  fresh findings review, the second
  run having been rate limited before
  it could post one. No Docstring
  Coverage item recurs on this pull
  request (no such check row stands
  in the shell's pre-merge table), so
  the dismissal basis of VL-46 and
  VL-51 is not needed here. Nothing
  on the pull request stands
  undisposed, and no source or prose
  change is owed to it. Any late
  findings on this round's pull
  request are owed to Round 9's
  contract by the same shape.
- VL-54. The three homes of the one
  line description, measured through
  the API at this round's base (main
  at bc6f944d, tree 26682ce9), never
  from a local clone. The GitHub
  repository description and the
  README lede carry one line verbatim:
  An experiment registry for a
  fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep
  or kill. Cargo.toml's description
  field carries the third variant
  VL-48 named: Every setup change as
  a tracked experiment, with a
  verdict. The standing precedent
  (secondtake's ST-51, onrecord's
  OR-50) permits exactly one move
  without David's word: where two
  homes carry one line verbatim and
  the third diverges, the third
  aligns to the standing line, byte
  for byte. The divergent home here
  is Cargo.toml, not the GitHub
  description (his surface), so the
  permitted move is the one taken:
  Cargo.toml's description now reads
  as the standing line, byte for
  byte, and no other home is edited.
  Storefront item 3 passes on this
  round's tree.
- VL-55. The temp-name draw, stated
  in its own terms before any
  measurement. The seam is
  write_atomically in src/cli.rs,
  which list and show reach when a
  view is written to an output path.
  The candidate is the temporary
  parent's bytes, the output name's
  bytes, a dot, and the suffix
  random_suffix draws, assembled as
  VL-43 contracts, created
  exclusively at mode 0600, and
  renamed onto the constructed path.
  The retry loop draws at attempt 0
  and increments on AlreadyExists
  while the attempt is below 16, so
  one write draws at most seventeen
  candidates, attempts 0 to 16. As
  shipped at this round's base,
  random_suffix renders eight
  lowercase hexadecimal characters
  from four bytes. Its primary arm
  reads those four bytes from
  /dev/urandom. Its fallback arm,
  where that read fails, folds a
  sixteen-byte little-endian mix by
  lanes to the same four bytes, the
  mix being the SystemTime
  nanoseconds xor the process id as
  a u128 shifted left by 64 xor the
  attempt, the fold VL-42 recorded.
  The two arms differ in kind: the
  primary arm's term is operating
  system randomness, which no
  observer can hold, and the
  fallback arm is a public function
  of a clock reading, the process
  id, and the attempt, every one of
  them knowable or fixable by an
  observer who can plant names in
  the output directory.
- VL-56. The adjudication, by
  measurement, in the sibling shape
  (tetlock's WW-70, unheard's UH-49,
  and bluepencil's BP-54), adapted to
  this draw's own budget of seventeen
  candidates. The battery (the round's
  scratch instrument) freezes the wall
  clock by an LD_PRELOAD interposition
  on clock_gettime at two readings,
  1700000000.123456789 and
  1800000000.987654321, fixes the
  process id in a fresh pid namespace
  at 1 and at 250 with genuineness
  verified from the pid file the
  child writes before it execs, and
  computes the fallback candidate set
  in advance over VL-55's fold. The
  baseline view document is 386
  bytes, taken unshimmed. On the
  shipped binary (a fresh build of
  this round's base tree, the
  control) the verdict splits by arm.
  The primary arm is acquitted: with
  /dev/urandom answering and all
  seventeen fallback candidates
  planted, at the first reading and
  pid 1 and at the second reading
  and pid 250 alike, the write lands
  and the document is byte-identical
  to the baseline, the plants
  standing inert, because the
  primary draw never reads the clock
  or the process id. The fallback
  arm is convicted: with every open
  of /dev/urandom failed by
  interposition with EACCES, the
  same seventeen plants refuse the
  write at both readings and both
  pids, exit 1 with the standing
  cannot write the view line naming
  the output, no output landed, and
  the directory left exactly as
  seeded, and sixteen plants write
  on the seventeenth candidate at
  both readings and both pids, the
  document byte-identical to the
  baseline. Under a frozen and known
  clock with its entropy source
  failed, the shipped draw is
  therefore the public function the
  sibling draws were, and a planted
  directory can deny the write
  outright in that arm. Correctness
  is never at stake (the create is
  exclusive, so a plant is never
  opened); the exposure is refusal
  only, and it is convicted because
  the fallback made a computable
  draw of what the Python's mkstemp
  and this draw's own primary arm
  keep unguessable.
- VL-57. The repair, in the fleet
  shape at the seam VL-55 states.
  random_suffix now draws sixteen
  bytes of operating system
  randomness fresh for each candidate
  from /dev/urandom through the
  standard library's file read, and
  renders the suffix from those bytes
  alone: the sixteen bytes fold by
  lanes to four (the fold the
  convicted fallback used, now fed by
  entropy instead of the clock mix),
  and each folded byte renders as two
  lowercase hexadecimal characters,
  so the suffix's shape, the
  candidate's construction and
  placement, the exclusive create,
  the 0600 mode, the seventeen
  candidate retry budget, the rename
  target, and the cannot-write text
  all stand unchanged. The failure
  posture is refusal, never fallback:
  where the entropy read fails or
  comes up short, and on the
  non-unix arm, which has no such
  source to read, the draw yields no
  suffix and write_atomically refuses
  through its standing arm, the
  cannot-write line at exit 1, before
  any create, so no computable draw
  is ever made silently. What the law
  now claims: the candidate set
  cannot be computed from the process
  id, nor from the process id and a
  frozen and known clock together,
  nor in the entropy source's failure
  arm, where no candidate is drawn at
  all. What it still does not claim:
  the suffix carries 32 bits, so a
  planted name can still collide by
  chance, once in 2^32 per candidate
  on average, and the redraw budget,
  not the draw, is what answers a
  collision; no collision-freedom is
  claimed. Three unit pins land with
  the repair in the shape WW-73
  settled: the entropy term decides
  the suffix (a change in any lane
  position of the entropy moves it),
  the suffix matches an independent
  derivation of the contracted fold,
  and a live draw renders eight
  lowercase hexadecimal characters or
  refuses, the pin accepting the
  contracted refusal where the source
  does not answer and never requiring
  two draws to differ.
- VL-58. The output guard,
  measured fresh at the shipped
  binary (a fresh build of this
  round's base tree) with a battery
  of nineteen cells over seeded
  registries. The guard resolves
  the output path and the registry
  path through the component walk
  and compares the finished paths.
  Naming the registry directly,
  naming it through a symlink,
  naming it through a dangling
  link whose target spelling
  normalizes onto it, naming it by
  an absolute path against a
  relative registry spelling all refuse
  before any write with the
  standing line, error, a view
  never overwrites the registry,
  at exit 1. A chain of links
  naming the registry refuses at
  lengths 1, 40, 100, 500, 990,
  and 991 and fails resolution at
  992 and 1000, exiting 1 with no
  output on either stream, the
  crash class standing since
  VL-13, so the walk's
  substitution budget splits
  exactly between 991 and 992 at
  this binary. A self-referential
  link fails resolution the same
  way. A hard link is a second
  path, not the registry path, so
  the guard passes it, the
  document lands at that name as
  its own file, and the registry
  bytes stand. A dangling link to
  a fresh name lands the document
  at the link's name and creates
  nothing at the target, and a
  symlinked directory component
  resolves through, the document
  landing at the real path. A
  dot-dot pops the resolved
  prefix, over a substituted
  loop, over the registry file
  component itself, and over a
  component that does not exist,
  so the guard passes those
  spellings, and the write then
  fails on the unenterable parent
  spelling with the cannot write
  the view line naming the output
  as spelled, nothing landed. A
  dead component with no dot-dot
  passes the guard and fails the
  same way. No cell wrote to the
  registry, and no repair is owed.
- VL-59. The registry load and
  check surface, measured at the
  same binary with nine crafted
  files. Check names every defect
  with its line, and list refuses
  each file with the registry has
  defects line carrying the same
  words. A duplicate opening of
  one experiment is named at its
  second line. A result event for
  an unopened experiment is
  named. A line that is not JSON
  is named not valid JSON, and a
  line carrying one key twice is
  refused at the same parse with
  the same words, so a duplicate
  key never folds silently. An
  opening whose baseline measures
  are absent is named baseline
  measures is not a non-empty
  object. A line that is a bare
  array is named the event is not
  an object, and an unknown type
  is named with the type itself.
  A recorded suggestion rewritten
  against its own measures is
  caught by recomputation and
  named the recorded suggestion
  does not match the measures it
  rests on, and a close resting
  on a line that is not its
  opening's result is named with
  the opening it fails to rest
  on. The surface is strict and
  exact, and no repair is owed.
- VL-60. The judgment band,
  measured at the same binary
  with thirteen cells driven by a
  scripted judge standing in for
  the pinned model. The seed's
  judged measure regresses past
  its margin, so the deterministic
  rule suggests rerun, as the
  card the judge receives states.
  Where the verdict equals the
  suggestion the judge is never
  consulted, no call is made even
  with a live judge configured
  whose every call fails, and the
  close records not consulted
  because the verdict and the
  suggestion agree. Contested
  with no judge configured, the
  close lands not consulted
  because no judge is configured.
  Contested with a judge, exactly
  three calls are made, the
  choice, the evidence score, and
  the regression risk score. A
  choice at confidence exactly
  0.6 lands judged, carrying the
  model name, the choice, the
  confidence, the probabilities,
  and both scores, and an integer
  confidence of 1 lands the same
  way. A confidence of 0.59
  refuses the close with the
  below 0.6 line and the registry
  byte-identical, an integer
  confidence of 0 refuses as
  below the floor, and an abstain
  refuses with the abstained
  line, likewise recording
  nothing. A failed call, an
  answer that is not JSON,
  probabilities that do not carry
  the four labels summing near
  one, a score outside its range,
  and an answer naming an
  unpinned model each land the
  close as unavailable with the
  reason stated in the record,
  and no call past the failing
  one is made. The band holds at
  every edge measured, and no
  repair is owed.
- VL-61. The gate and the
  reruns on the final tree of
  this round. Cargo test passes
  47, none failed, the 44 of the
  base plus the three pins of
  VL-57 (24 judgment, 20
  registry, 3 library). Cargo
  clippy over all targets with
  warnings denied is clean, on
  the tree where the VL-57 test
  module stands at the end of
  src/cli.rs, moved there
  byte-identical because
  clippy's items after test
  module rule refused its first
  placement mid-file. Cargo fmt
  check is clean, and the
  codepoint scan stands at zero
  of U+2014 and zero of U+2013.
  The draw battery reruns whole
  against the repaired binary,
  13 cells and none failed, the
  control and repaired baseline
  documents both 386 bytes and
  byte-equal, and the repaired
  cells R1 to R4 passing as
  VL-57 contracts. The
  differential harness against
  the final Python oracle
  completed 144 of its 198
  cases clean in its best run
  on this tree and was stopped
  there by the box write fault
  refusing an append, not by a
  divergence, and its full run
  stands owed to the record.
- VL-62. README records 8 of 100 only
  in the closure commit written after
  this round's three reviews, fixes,
  and owner verification.
- VL-63. The full differential
  harness run VL-61 left owed is
  discharged. The red team ran it
  against the final tree's binary
  in its own review, 198 cases and
  400 steps, all byte-clean against
  the Python oracle, on the first
  attempt in an open write window.
  The run the lane banked at 144
  cases stands as the partial it
  was, and no case in either run
  diverged.

## Round 9

- VL-64. Round 9 opens with the full
  read-back of pull request 12 (Round
  8), taken on 2026-11-10 through the
  API, every review body to its end,
  every review comment thread, and
  every issue comment, the follow-up
  shells included. Inline review
  comments, none. Reviews, one:
  CodeRabbit's COMMENTED review
  5821200018, submitted
  2026-11-09T03:01:05Z, over the range
  from bc6f944d (the base) to
  4b1390f4 (the closure head), its
  files CONTRACT.md, Cargo.toml,
  README.md, and src/cli.rs, its run
  configuration stating that no
  configuration file from this
  repository was applied, profile
  CHILL, layers defaults. The body
  posts no inline findings; its one
  substantive item is the failed
  pre-merge check named in its prompt
  block, Docstring Coverage. Issue
  comments, five: the summary shell
  6731146073, standing in its final
  form as the rate-limit banner (the
  next included review available in
  45 minutes, its second run header
  over the range from 4b1390f4 to
  e4cc5d3d) over a complete
  walkthrough of the first head, with
  Merge Risk Minimal up to 4b1390f4
  and its pre-merge checks at four
  passed (Description, Title, Linked
  Issues, and Out of Scope Changes)
  and one failed, Docstring Coverage
  at 40.00 percent against the
  required 80.00 percent, analyzed 5
  functions across 1 file with 3
  skipped as unsupported, naming no
  function; the owner's trigger
  6731311511; the action notice
  6731312811 (Action not completed,
  review rate limited, carrying no
  finding); the owner's disposition
  6731362254; and the owner's landing
  comment 6732476919. The one item
  stands disposed in 6731362254 on
  the standing basis of VL-51 and
  ST-61, and this round verified the
  basis limb by limb firsthand: the
  shell names no function (it counts
  five and names none); this
  repository carries no docstring
  configuration or threshold of its
  own (no configuration file for the
  review tool, nothing in Cargo.toml
  or the CI workflow, and the
  review's own run configuration
  confirms none was applied); and the
  check applies the review tool's
  default profile, so there is no
  repository rule for it to enforce.
  No review comment thread exists on
  the pull request, so no
  acknowledgment reply rides one, and
  nothing arrived after the landing
  comment. The confirmation Round 8's
  spec notes owed is recorded here:
  no later driver landed a fuller
  harness run than the red team's,
  nothing in this pull request's
  record or on main since shows one,
  and the run VL-63 records stands as
  the complete run of record for
  Round 8. Nothing on the pull
  request stands undisposed, and no
  source or prose change is owed to
  it. Any late findings on this
  round's pull request are owed to
  Round 10's contract by the same
  shape.
- VL-65. The judgment band, measured
  fresh at the shipped binary (a
  fresh build of this round's base
  tree) with a battery of fifteen
  cells driven by a scripted judge
  over the live transport. The seed's
  judged measure regresses, so the
  deterministic rule suggests rerun.
  Where the verdict equals the
  suggestion the judge is never
  consulted, no call made even with a
  live judge configured whose every
  call fails, and the close records
  not consulted because the verdict
  and the suggestion agree. Contested
  with no judge configured, the close
  lands not consulted because no
  judge is configured. Contested with
  a judge, exactly three calls are
  made, the choice, the evidence
  score, and the regression risk
  score. A choice at confidence
  exactly 0.6 lands judged, carrying
  the model name jev-1.13.0, the
  choice, the confidence, the
  probabilities over the four labels,
  both scores, the card, and the
  card's SHA-256 digest. A confidence
  of 0.599999 refuses the close after
  the three calls with the line that
  the judge's confidence 0.599999 is
  below 0.6, the registry
  byte-identical, and a confidence of
  0.600001 lands judged. An integer
  confidence of 1 lands judged,
  recorded as the integer 1, and an
  integer confidence of 0 refuses as
  below the floor, the registry
  byte-identical. An abstaining
  choice at confidence 0.99 refuses
  with the abstained line, the
  registry byte-identical, and an
  abstaining judge under an agreement
  verdict is never consulted at all.
  The unavailable family lands the
  close with its reason recorded and
  no call past the failing one: a
  failed call (one call), an answer
  that is not JSON (one call),
  probabilities carrying the four
  labels but summing to one half, so
  the choice answer is malformed (one
  call), an evidence score of 11,
  outside its range, so the evidence
  answer is malformed (two calls),
  and an answer naming the unpinned
  model jev-1.12.0 (one call). A keep
  close naming a successor with a
  live judge configured refuses on
  the successor rule, at line 3 of
  the registry, with no judge call
  made and the registry
  byte-identical: the close's own
  refusals precede any call. The band
  holds at every edge measured, and
  no repair is owed.
- VL-66. The hearing surfaces at a
  realistic shape, measured against
  the Python oracle (the final Python
  tree at d62e5dc, consumed as the
  renamed copy the differential
  harness uses; the copy's fidelity
  is proven by the harness run
  VL-69 records). A corpus of 48
  experiments is built twin-style,
  the same steps driven at both
  implementations: sources cycling
  eight fleet names, two to three
  judged measures per experiment with
  margins on some entries, one to
  three results each, verdicts mixed
  by construction. The suggestion is
  steered under the VL-9 rule (moves
  of 30 percent against margins of at
  most 10, the default included) and
  verified against the recorded
  suggestion at every close. The
  count: 48 opens, 92 results, 40
  closes, 180 steps, every step's
  exit code, stderr, and registry
  bytes identical, the registry bytes
  compared after the prescribed
  judgment cut, each cut span
  verified to be exactly one of the
  two not consulted shapes, keys in
  order, before its removal; the
  final check and the final list are
  identical without any cut beyond
  the same spans in the list. Sixteen
  closes agree with the suggestion,
  twenty-four are contested, and
  eight experiments stand open. A
  judged pass over the eight
  contested opens, on the Rust side
  with the scripted judge rotating
  through its arms, lands two judged
  (one of them a rerun verdict whose
  named successor stands recorded in
  the close), refuses four with the
  registry byte-identical (two below
  the confidence floor, two
  abstentions), and lands two
  unavailable (a failed call, a
  malformed evidence answer), the
  call sequences exact per arm, and
  the final registry reads sound
  under check. No repair is owed.
- VL-67. The temp draw, measured
  fresh at the same binary after the
  VL-57 repair, with a battery of
  eight cells over a seeded registry
  whose baseline view document is 168
  bytes. The entropy failure arm,
  every open of /dev/urandom failed
  by interposition with EACCES,
  refuses with the cannot write the
  view line naming the output at exit
  1, the directory exactly as seeded.
  The short-read arm, opens serving
  five bytes and then end of file so
  the sixteen-byte read comes up
  short, refuses the same way:
  refusal, never a fallback, on both
  arms, before any create. The clock
  plants stand inert: all seventeen
  candidates of the convicted
  pre-repair fallback, computed from
  VL-55's fold, planted at the frozen
  reading 1700000000.123456789 in a
  pid namespace at pid 1 and at
  1800000000.987654321 at pid 250,
  the pids genuine from the pid and
  NSpid files the runs record, and at
  both the write lands and the
  document is byte-identical to the
  baseline, because the repaired draw
  never reads the clock or the
  process id. The plant bound is
  proven by a positive control, not
  assumed: with opens of /dev/urandom
  serving a deterministic byte
  stream, one block per open in
  order, whose folded suffixes the
  battery computes independently of
  the crate, sixteen planted
  candidates let the write land on
  the seventeenth candidate, and
  seventeen planted candidates refuse
  it with the directory as seeded, so
  the candidate set the draw walks
  and the seventeen-candidate budget
  are both measured at the shipped
  binary. A first candidate planted
  and the second draw served short
  refuses mid-loop, before any create
  on the short draw. No repair is
  owed.
- VL-68. The resolver walk, measured
  fresh at the same binary at its own
  constant, MAX_WALK_DEPTH 991 in
  src/cli.rs, a number belonging to
  this CLI's stack as VL-36 measured
  it and never inherited, with a
  battery of eight cells. A chain of
  links naming the registry refuses
  with the standing line, error, a
  view never overwrites the registry,
  at lengths 990 and 991, and at 992
  fails resolution silently, exit 1
  with no output on either stream and
  the directory unchanged, the crash
  class standing since VL-13. A chain
  of 991 links to a fresh name
  completes inside the budget: the
  document lands at the spelled path
  byte-identical to the baseline, and
  nothing lands at the target. A
  chain of 992 links to a fresh name
  fails the same silent way, so the
  budget is the walk's, not the
  guard's. A self-referential link
  named as the output path fails
  resolution silently the same way. A
  loop the finished path pops, the
  output spelled through a
  self-referential link and a
  dot-dot, passes the guard, the
  finished path being clean, and the
  write then fails on the
  unenterable spelling with the
  cannot write the view line naming
  the output as spelled, nothing
  landed: the VL-58 posture, measured
  here on both twins, the oracle and
  the port refusing identically. A
  link met again after its resolution
  completed is followed again: the
  spelling L/../L/view.json lands the
  document at the finished path. No
  repair is owed.
- VL-69. The gate and the reruns on
  the final tree of this round.
  Cargo test passes 47, none failed
  (24 judgment, 20 registry, 3
  library). Cargo clippy over all
  targets with warnings denied is
  clean. Cargo fmt check is clean,
  and the codepoint scan stands at
  zero of U+2014 and zero of U+2013.
  The batteries of VL-65 to VL-68 run
  whole against the round's binary:
  the judgment battery at fifteen
  cells, the draw battery at eight,
  the walk battery at eight, and the
  corpus of VL-66 at 180 twin steps
  and eight judged closes, none
  failed. The differential harness
  against the final Python oracle ran
  whole on the first attempt: 198
  cases and 400 steps, all byte-clean.
  This round changes no source, so
  the binary the batteries and the
  harness measured, built fresh from
  this round's base tree, is the
  final tree's binary. Item 3 stands
  passing on this round's own
  evidence, measured through the API
  at the round's base: the repository
  description, the README lede, and
  the Cargo.toml description field
  carry the one line byte for byte,
  as VL-54 left them.
- VL-70. README records 9 of 100 only
  in the closure commit written after
  this round's three reviews, fixes,
  and owner verification.
- VL-71. Round 10 opens with the full
  read-back of pull request 13 (Round
  9), taken on 2026-11-11 through the
  API, every review body to its end,
  every review comment thread, and
  every issue comment. Reviews, none.
  Inline review comments, none. Issue
  comments, four: the CodeRabbit
  summary shell 6761037568, created
  2026-11-10T06:49:45Z and edited into
  its final form at 06:55:53Z; the
  owner's trigger 6761059805; the
  bot's action notice 6761128171
  (Action not completed, already
  reviewed the last commit, carrying
  no finding); and the owner's landing
  comment 6763574496. The shell in its
  final form states that no actionable
  comments were generated in the
  recent review, carries a complete
  walkthrough of the change, base to
  the closure head 31158973, Merge
  Risk Minimal, and pre-merge checks
  at exactly four passed: Description,
  Title, Linked Issues, and Out of
  Scope Changes. No Docstring Coverage
  row stands anywhere in the shell's
  final form, and no review object
  stands on the pull request at all.
  This contracts exactly the one item
  Round 9's record left open: the
  standards review's Minor against the
  Round 9 build report, whose
  CodeRabbit evidence stated five
  passed pre-merge checks including a
  Docstring Coverage pass, where the
  shell lists four and no Docstring
  row. The Minor stands confirmed
  against the report and against no
  pull request artifact; the
  nothing-to-fix disposition was
  correct on the shell's own words,
  and the shell's run configuration
  (no configuration file from this
  repository applied, profile CHILL)
  stands as the shell itself records
  it. No acknowledgment reply rides
  any thread, because no review
  comment thread exists, and nothing
  arrived after the landing comment.
  No Round 9 disposition is falsified
  by this record beyond the shell-form
  point the Minor already recorded,
  nothing on the pull request stands
  undisposed, and no source or prose
  change is owed to it. Any late
  findings on this round's pull
  request are owed to Round 11's
  contract by the same shape.
- VL-72. The judgment band, measured
  fresh at the shipped binary (a fresh
  build of this round's base tree)
  with a battery of seventeen cells
  driven by a scripted judge over the
  live transport and the fixture
  transport. The seed's judged measure
  regresses, so the deterministic rule
  suggests rerun. Where the verdict
  equals the suggestion the judge is
  never consulted, no call made even
  with a live judge configured whose
  every call fails, and the close
  records not consulted because the
  verdict and the suggestion agree.
  Contested with no judge configured,
  the close lands not consulted
  because no judge is configured.
  Contested with a judge, exactly
  three calls are made, the choice,
  the evidence score, and the
  regression risk score. A choice at
  confidence exactly 0.6, decided at
  the point itself and not bracketed
  from its neighbours, lands judged,
  carrying the model name jev-1.13.0,
  the choice, the confidence, the
  probabilities over the four labels,
  both scores, the card, and the
  card's SHA-256 digest. A confidence
  of 0.599999 refuses the close after
  the three calls with the line that
  the judge's confidence 0.599999 is
  below 0.6, the registry
  byte-identical, and a confidence of
  0.600001 lands judged. An integer
  confidence of 1 lands judged,
  recorded as the integer 1, and an
  integer confidence of 0 refuses as
  below the floor, the registry
  byte-identical. An abstaining choice
  at confidence 0.99 refuses with the
  abstained line, the registry
  byte-identical, and an abstaining
  judge under an agreement verdict is
  never consulted at all. The abstain
  posture holds in the fixture seat as
  well: an abstain replayed from a
  judgments file keyed by the recorded
  card's digest refuses the same close
  with the registry byte-identical,
  and the recorded responses replayed
  whole through the fixture land a
  judged close byte-identical to the
  live seat's. The unavailable family
  lands the close with its reason
  recorded and no call past the
  failing one: a failed call (one
  call), an answer that is not JSON
  (one call), probabilities carrying
  the four labels but summing to one
  half, so the choice answer is
  malformed (one call), an evidence
  score of 11, outside its range, so
  the evidence answer is malformed
  (two calls), and an answer naming
  the unpinned model jev-1.12.0 (one
  call). A keep close naming a
  successor with a live judge
  configured refuses on the successor
  rule, at line 3 of the registry,
  with no judge call made and the
  registry byte-identical: the close's
  own refusals precede any call. The
  band holds at every edge measured,
  and no repair is owed.
- VL-73. The band's outer edges,
  measured fresh at the same binary
  with a battery of five cells at the
  margin's exact points. The
  regression rule is strict: a move
  regresses only past the margin,
  never at it. The points are chosen
  exact in binary, a baseline of 128
  and a margin of 12.5, so the point
  itself is decided and not bracketed.
  A move of 16, the scaled move
  exactly 12.5, does not regress: the
  suggestion is keep, a rerun verdict
  against it is contested and consults
  the judge, and a keep verdict agrees
  and consults nothing, even with a
  failing judge configured. A move of
  17, the scaled move 13.28125,
  regresses: the suggestion is rerun
  and a rerun verdict agrees with zero
  calls. With two judged measures,
  both standing exactly at their
  margins (a rise of 16 against a
  lower direction, a fall of 16
  against a higher direction), the
  suggestion is keep; both one unit
  past their margins, the suggestion
  is kill, the regressed list naming
  both measures. The edges hold at
  their points, and no repair is owed.
- VL-74. The hearing surfaces at a
  realistic shape, measured against
  the Python oracle (the final Python
  tree at d62e5dc, consumed as the
  renamed copy the differential
  harness uses; the copy's fidelity is
  proven by the harness run VL-76
  records). A corpus of 120
  experiments is built twin-style, the
  same steps driven at both
  implementations: sources cycling
  eight fleet names, two to three
  judged measures per experiment, half
  the judged entries carrying a margin
  of 5 or 10 and a quarter of the
  openings an event-level margin of
  10, one to twelve results each,
  eight experiments carrying twelve
  results as deep histories. The
  suggestion is steered by
  construction (improvement inside the
  margins for keep, a 30 percent move
  on one judged measure for rerun, on
  two for kill) and verified against
  the recorded suggestion at every
  close under the VL-9 rule,
  recomputed in exact rational
  arithmetic by the driver. The count:
  120 opens, 292 results, 100 closes,
  517 steps in all (the 512 events and
  the five refusal steps below), every
  step's exit code,
  stdout, and stderr identical, the
  registry bytes compared after the
  prescribed judgment cut, each cut
  span verified to be exactly one of
  the two not consulted shapes before
  its removal, and the final list and
  show compared after the same cut in
  their pretty form; the final check
  is identical and sound. Sixty closes
  agree with the suggestion, forty are
  contested, and twenty experiments
  stand open. The registry at the
  corpus end stands at 512 lines and
  113,545 bytes on the Rust side. Five
  refusal steps ride the corpus, each
  refused identically on both twins: a
  show naming a missing experiment, a
  result for an unopened experiment, a
  duplicate opening, a second close of
  a closed experiment, and a keep
  close naming a successor against
  that same closed experiment, which
  refuses on the already closed line
  on both twins; the successor rule
  itself is exercised against an open
  experiment in VL-72. A judged pass
  over twelve of the contested opens,
  on the Rust side with the scripted
  judge rotating through six arms
  twice, lands four judged (two at
  confidence 0.9, two at exactly 0.6),
  refuses four with the registry
  byte-identical (two at 0.59, two
  abstentions), and lands four
  unavailable (two failed calls, two
  malformed evidence answers), and the
  final registry, 520 lines and
  120,515 bytes, reads sound under
  check. One earlier run of this corpus
  died mid-corpus when the shared /tmp
  scratch filled and a registry append
  met ENOSPC, truncating a line and
  poisoning the run's remaining steps;
  the poisoned registry was discarded
  whole, and the clean re-run after
  space was freed is the run of record
  above; the killed run is counted
  neither for nor against. The round's
  scratch and instruments staged in
  /tmp, as VL-78 records. No behavioral
  repair is owed.
- VL-75. The resolver walk and the
  flighttape pin, measured fresh at
  the same binary. The walk battery
  runs eight cells at this CLI's own
  constant, MAX_WALK_DEPTH 991 in
  src/cli.rs. A chain of links naming
  the registry refuses with the
  standing line, error, a view never
  overwrites the registry, at lengths
  990 and 991, and at 992 fails
  resolution silently, exit 1 with no
  output on either stream and the
  directory unchanged, the crash class
  standing since VL-13. A chain of 991
  links to a fresh name completes
  inside the budget: the document
  lands at the spelled path
  byte-identical to the baseline, and
  nothing lands at the target. A chain
  of 992 links to a fresh name fails
  the same silent way, so the budget
  is the walk's, not the guard's. A
  self-referential link named as the
  output path fails resolution
  silently the same way. A loop the
  finished path pops, the output
  spelled loop/../view.json, passes
  the guard, the finished path being
  clean, and the write then fails on
  the unenterable spelling with the
  cannot write the view line naming
  the output as spelled, nothing
  landed, measured on both twins, the
  oracle and the port refusing
  identically. A link met again after
  its resolution completed is followed
  again: the spelling
  dirlink/../dirlink/view.json lands
  the document at the finished path.
  The pin stands as Cargo.lock
  resolves it: the flighttape git
  dependency at exactly
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  whose tree through the API is
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at exactly 50 blobs, the golden
  fixture among them at blob
  41e713560548837a3f406f1bb23cb21eaef74ad7,
  hash-identical in the cargo checkout
  at that revision. The pinned crate's
  own identity and digest suites run
  at the checkout (identity 13 tests,
  digest_cli 5, none failed),
  executing the golden
  digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  over the fixture. No repair is owed.
- VL-76. The gate and the reruns on
  the final tree of this round. Cargo
  test passes 47, none failed (24
  judgment, 20 registry, 3 library).
  Cargo clippy over all targets with
  warnings denied is clean. Cargo fmt
  check is clean, and the codepoint
  scan stands at zero of U+2014 and
  zero of U+2013. The batteries of
  VL-72 to VL-75 run whole against the
  round's binary: the judgment battery
  at twenty-two cells (seventeen in
  VL-72, five in VL-73), the walk
  battery at eight, and the corpus of
  VL-74 at 517 twin steps and twelve
  judged closes, none failed. The
  differential harness against the
  final Python oracle ran whole on the
  first attempt: 198 cases and 400
  steps, all byte-clean. This round
  changes no source, so the binary the
  batteries and the harness measured,
  built fresh from this round's base
  tree, is the final tree's binary.
  Item 3 stands passing on this
  round's own evidence, measured
  through the API at the round's base:
  the repository description, the
  README lede, and the Cargo.toml
  description field carry the one line
  byte for byte, as VL-54 left them.
- VL-77. README records 10 of 100 only
  in the closure commit written after
  this round's three reviews, fixes,
  and owner verification.

- VL-78. The repair record. The spec
  fidelity review returned NOT PASS
  with two Majors and two Minors, and
  the standards review APPROVE with
  three Minors; this commit repairs
  the record and changes no source.
  Major one: VL-74's step count read
  520, where its enumerated events sum
  to 512, corroborated by the
  corpus-end registry at 512 lines;
  with the five refusal steps the true
  count is 517 twin steps, and the 520
  in VL-74 is the final registry's
  line count after the judged pass
  (512 events and the eight landing
  judged closes), a line count VL-74
  already states correctly in its own
  registry sentence. VL-74 and VL-76
  are restated to 517, and the pull
  request body with them. Major two:
  VL-75 named the pinned crate's
  suites as 5 and 13 in identity and
  digest order; counted at the pinned
  commit, the identity suite carries
  13 test functions and digest_cli 5,
  and VL-75 and the body are restated.
  Minor one: VL-71 called the PR 13
  shell's walkthrough a walkthrough of
  the slice head; the shell covers
  the change from the base to the
  closure head 31158973, and VL-71 is
  restated. Minor two: VL-74 carried
  no mention of the earlier corpus
  run killed when the shared /tmp
  scratch filled (ENOSPC); the
  disclosure now stands in VL-74
  itself. The standards Minors: the
  body's storefront item 1 credited
  the README with release, license,
  and Rust edition badges where the
  head README carries the CI badge
  alone, and the body is corrected;
  the corpus arithmetic is Major one
  above; and neither VL-71 to VL-77
  nor the body disclosed that the
  round's scratch, instruments, and
  lane reports staged in /tmp under
  the workspace file-count quota.
  The staging was wiped by a box
  replacement after the pull request
  opened, and this item is the
  disclosure. What survives stands on
  the remote (the contract, the body,
  the hosted gate on the exact head)
  and on the owner's independent
  re-verification, including its own
  gate re-run at 47 and its own judged
  cells at the floor's exact points.
  A second review set seated by a
  parallel owner instance after the
  same restart returned standards
  APPROVE with the badge Minor,
  converging with this set; its spec
  report was overwritten on disk
  before its verdict reached this
  instance and is recorded as not
  recovered.
- VL-79. Round 11 opens with the full
  read-back of pull request 14 (Round
  10), taken on 2026-11-12 through the
  API, every review body to its end,
  every review comment thread, and every
  issue comment. Reviews, none. Inline
  review comments, none. Issue comments,
  seven: the CodeRabbit summary shell
  6792096226, created
  2026-11-11T09:23:44Z and edited into
  its final form at
  2026-11-11T10:02:13Z; the owner's
  trigger 6792099920; the bot's action
  notice 6792193593 (Action not
  completed, already reviewed the last
  commit, carrying no finding); the
  owner's landing comment 6792906001;
  the second owner instance's re-audit
  and delta record 6793156108; the third
  owner seat's concurrence 6793726559;
  and the third seat's red team verdict
  6793880034. Nothing has arrived on the
  pull request after
  2026-11-11T10:52:54Z. The shell in its
  final form states that no actionable
  comments were generated in the recent
  review, carries a complete walkthrough
  of the change, Merge Risk Minimal, and
  pre-merge checks at exactly four
  passed: Title, Linked Issues, Out of
  Scope Changes, and Description. No
  Docstring Coverage row stands anywhere
  in the shell's final form, and no
  review object stands on the pull
  request at all. The shell assesses the
  closure head
  36765f0c46c912554aa374e96875db0ca594cbca
  and never the repair head
  8cfc88e921c313227d1a00d01a5a9818d86435ee:
  its change assessment commit, its
  reviewed commit range (the base
  0d565de5 to 36765f0c), and its merge
  risk coverage all name the closure
  head, the head the round's
  dispositions were written against. The
  union of the seats' dispositions
  grades as follows. The merging seat's
  reviews returned spec fidelity NOT
  PASS with two Majors and two Minors,
  standards APPROVE with three Minors,
  and its own red team APPROVE; both
  Majors were true and stand repaired in
  the repair head, recorded in VL-78:
  VL-74's step count read 520 where its
  enumerated events sum to 512 (120
  opens, 292 results, 100 closes), the
  true count with the five refusal steps
  being 517, and VL-75 named the pinned
  crate's suites as 5 and 13 in identity
  and digest order where the counts are
  identity 13 and digest_cli 5. The
  second seat's delta reviews returned
  spec fidelity NOT PASS with one Minor,
  against VL-78's closing sentence
  recording its spec report as not
  recovered, and standards APPROVE with
  three Nits; the Minor stands
  discharged by that seat's own
  acceptance on the record, the
  qualification in the pull request
  body's second owner instance note
  together with comment 6793156108, no
  measurement or figure is affected, and
  no tree change was owed. The third
  seat's red team verdict stands at
  6793880034: APPROVE with zero
  Critical, zero Major, and one Minor,
  the suite pairing in VL-75, since
  repaired in the repair head and
  verified by that lane at the merged
  head. One record state stands exactly
  as the live record shows it and is not
  discharged: the second seat's red team
  lane was still running when the merge
  landed (6793156108), and no comment
  since has recorded its verdict or
  converted its held position. The
  sibling report that seat's lane found
  occupying its assigned report path, a
  NOT APPROVE on coverage grounds alone
  whose own corpus the environment
  destroyed, stands described inside
  6793880034, and the clean corpus
  executions that verdict's erratum
  prescribed are among the executions
  6793880034 records; the conversion
  itself was never written on the pull
  request by the seat that owed it, and
  this round does not write it for them.
  The same comment charged this round's
  read-back with discharging six
  exploratory band cells by execution;
  VL-80 discharges them. No Round 10
  disposition is falsified by this
  record, nothing on the pull request
  stands undisposed beyond the
  conversion state just named, and no
  source or prose change is owed to it.
  Any late findings on this round's pull
  request are owed to Round 12's
  contract by the same shape.
- VL-80. The judgment band, measured
  fresh at the shipped binary (a fresh
  build of this round's base tree) with
  a battery of twenty-four cells driven
  by a scripted judge over the live
  transport and the fixture transport;
  no live model was called. The seed's
  measures stand even, so the
  deterministic rule suggests keep and a
  rerun verdict stands contested. Where
  the verdict equals the suggestion the
  judge is never consulted, no call made
  even with a live judge configured
  whose every call fails, and the close
  records not consulted because the
  verdict and the suggestion agree.
  Contested with no judge configured,
  the close lands not consulted because
  no judge is configured. Contested with
  a judge, exactly three calls are made,
  the choice, the evidence score, and
  the regression risk score, except
  where a malformed or failed answer
  ends the consultation at the failing
  call. A choice at confidence exactly
  0.6, decided at the point itself and
  not bracketed from its neighbours,
  lands judged, carrying the model name
  jev-1.13.0, the choice, the
  confidence, the probabilities over the
  four labels, both scores, the card,
  and the card's SHA-256 digest. A
  confidence of 0.599999 refuses the
  close after the three calls with the
  line that the judge's confidence
  0.599999 is below 0.6, the registry
  byte-identical; a confidence of
  0.600001 lands judged; the
  fifteen-nines spelling
  0.599999999999999 refuses the same
  way. An integer confidence of 1 lands
  judged, recorded as the integer 1, and
  an integer confidence of 0 refuses as
  below the floor, the registry
  byte-identical. The 6e-1 spelling of
  0.6 lands judged and records 0.6.
  Scores stand inside their bounds at
  the bounds themselves: an evidence
  score of 10 and a risk score of 0 land
  judged, and a risk score of 10.5 lands
  the close unavailable at the third
  call with the risk answer malformed.
  An abstaining choice refuses with the
  abstained line and the registry
  byte-identical at confidence 0.99, at
  exactly 0.6, and at 0. In the fixture
  seat, the recorded responses replayed
  whole land a judged close whose
  registry is byte-identical to the live
  seat's, and an abstain replayed from
  the same judgments file refuses the
  same close. The unavailable family
  lands the close with its reason
  recorded and no call past the failing
  one: a failed call (one call), an
  answer that is not JSON (one call), an
  answer naming the unpinned model
  jev-1.12.0 (one call), probabilities
  carrying the four labels but summing
  to one half, so the choice answer is
  malformed (one call), and an evidence
  score of 11, outside its range, so the
  evidence answer is malformed (two
  calls). A kill close naming a
  successor against a regressed seed,
  with a live judge configured, refuses
  on the successor rule at line 3 of the
  registry with no judge call made and
  the registry byte-identical: the
  close's own refusals precede any call.
  The band's outer edges stand at their
  exact points in two further cells: a
  move of 16 on a baseline of 128, the
  scaled move exactly 12.5 against a
  margin of 12.5, does not regress and
  the suggestion is keep; a move of 17,
  the scaled move 13.28125, regresses
  and the suggestion is rerun. Six of
  these cells discharge by execution the
  charge comment 6793880034 left on pull
  request 14: the abstain at exactly
  0.6, the abstain at 0, the
  probabilities summing to one half, the
  unpinned model, the answer that is not
  JSON, and the evidence score of 11.
  The band holds at every point
  measured, and no repair is owed.
- VL-81. The resolver walk and the
  flighttape pin, measured fresh at the
  same binary against the Python oracle
  (the final Python tree at d62e5dc, its
  six Python files verified blob for
  blob against that commit through the
  API before the run). The walk battery
  runs ten cells at this CLI's own
  constant, MAX_WALK_DEPTH 991 in
  src/cli.rs: the output path is spelled
  through a chain of directory links so
  that it resolves to the registry
  itself, six cells on the list surface
  (chain lengths 990, 991, and 992 on
  both implementations) and four on the
  show surface (991 and 992 on both). At
  990 and at 991 links the guard refuses
  with the standing line, error, a view
  never overwrites the registry, on both
  implementations. At 992 links the
  resolution fails on both: the port
  exits 1 with no output on either
  stream, the crash class standing since
  VL-13, and the oracle raises
  RecursionError out of resolve() and
  exits 1. The split stands at exactly
  991 and 992, and the budget is the
  walk's own. The pin stands as
  Cargo.lock resolves it: the flighttape
  git dependency at exactly
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  verified live through the API in this
  round, whose tree is
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at exactly 50 blobs, the golden
  fixture among them at blob
  41e713560548837a3f406f1bb23cb21eaef74ad7
  (tests/fixtures/claude_session_synthetic.jsonl).
  The pinned crate's own identity and
  digest suites run at the cargo
  checkout (identity 13 tests,
  digest_cli 5, none failed), executing
  the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  over the fixture. No repair is owed.
- VL-82. The hearing surfaces at a
  realistic shape, measured against the
  Python oracle (the final Python tree
  at d62e5dc, verified as VL-81
  records). A corpus of 126 experiments
  is built twin-style, the same steps
  driven at both implementations:
  sources cycling eight fleet names, two
  or three judged measures per
  experiment, half the judged entries
  carrying a margin of 5 or 10 and a
  quarter of the openings an event-level
  margin of 10, one to twelve results
  each, ten experiments carrying twelve
  results as deep histories. The
  suggestion is steered by construction
  (improvement inside the margins for
  keep, a 30 percent move on one judged
  measure for rerun, on two for kill)
  and verified against the recorded
  suggestion at every close under the
  VL-9 rule, recomputed in exact
  rational arithmetic by the driver,
  with no disagreement. The count: 126
  opens, 801 results, 105 closes, 1,037
  steps in all (the 1,032 events and the
  five refusal steps below), every
  step's exit code, stdout, and stderr
  identical, the registry bytes compared
  after the prescribed judgment cut,
  each cut span verified to be exactly
  one of the two not consulted shapes
  before its removal, and the final list
  and show compared after the same cut
  in their pretty form (105 judgment
  spans cut on the port's list, none on
  the oracle's, which carries no
  judgment member); the final check is
  identical and sound. Sixty-three
  closes agree with the suggestion,
  forty-two are contested, and
  twenty-one experiments stand open. The
  registry at the corpus end stands at
  1,032 lines and 216,165 bytes on the
  Rust side. Five refusal steps ride the
  corpus, each refused identically on
  both twins: a show naming a missing
  experiment, a result for an unopened
  experiment, a duplicate opening, a
  second close of a closed experiment,
  and a keep close naming a successor
  against that same closed experiment,
  which refuses on the already closed
  line on both twins; the successor rule
  itself is exercised against an open
  experiment in VL-80. A judged pass
  over twelve contested opens, on the
  Rust side with the scripted judge
  rotating through six arms twice, lands
  four judged (two at confidence 0.9,
  two at exactly 0.6), refuses four with
  the registry byte-identical (two at
  0.59, two abstentions), and lands four
  unavailable (two failed calls, two
  malformed evidence answers), and the
  final judged registry, 32 lines and
  13,419 bytes, reads sound under check.
  No behavioral repair is owed.
- VL-83. The gate on the final tree of
  this round. Cargo test passes 47, none
  failed (24 judgment, 20 registry, 3
  library). Cargo clippy over all
  targets with warnings denied is clean.
  Cargo fmt check is clean, and the
  codepoint scan stands at zero of
  U+2014 and zero of U+2013 over the
  tree, and over all 21 blobs read back
  through the API at the head this gate
  ran on. The batteries of VL-80 to
  VL-82 run whole against the round's
  binary: the judgment battery at
  twenty-four cells, the walk battery at
  ten, and the corpus of VL-82 at 1,037
  twin steps with its judged pass over
  twelve contested opens, none failed. This round changes no
  source, so the binary the batteries
  measured, built fresh from this
  round's base tree, is the final tree's
  binary. Item 3 stands passing on this
  round's own evidence, measured through
  the API at the round's base: the
  repository description, the README
  lede, and the Cargo.toml description
  field carry the one line byte for
  byte, as VL-54 left them. The round's
  staging stands disclosed: this is the
  round's third seating, the first two
  seatings having died silently in box
  restarts, the first leaving nothing
  and the second leaving only the remote
  branch standing at the base while its
  /tmp scratch was wiped whole. This
  seating staged its scratch and
  instruments in /tmp under the
  workspace file-count quota and landed
  every commit server-side in small
  pieces as each family finished,
  asserting the pushed tree SHA at every
  step, so the remote branch is the
  round's bank.

- VL-84. README records 11 of 100 only
  in the closure commit written after
  this round's three reviews, fixes, and
  owner verification.

- VL-85. The repair record. CodeRabbit's
  step 0 review on pull request 15
  (review 5876639927; one actionable
  comment, 4552122144, a Minor against
  VL-83) is true, and this commit
  repairs it. VL-83 counted the judged
  pass as twelve judged closes, where
  VL-82 records twelve contested opens,
  of which eight closes landed (four
  judged and four unavailable) and four
  were refused with the registry
  byte-identical. VL-83 is restated in
  this commit to name the judged pass
  over twelve contested opens, and the
  pull request body's verified block is
  corrected with it. The shell's own
  merge risk note names the same fix.
  The repair changes no other figure, no
  measurement, and no source.
- VL-86. The second repair record, on
  the owner's three reviews of pull
  request 15. The spec fidelity review
  returned NOT PASS with two Minors and
  the standards review NOT APPROVE with
  one Minor; the red team returned
  APPROVE with no findings. Two record
  defects stand between them. First,
  VL-80's discharge list misnames one
  of the six cells comment 6793880034
  charged on pull request 14: the
  charge names a risk score of 11,
  where the list names the evidence
  score of 11. VL-80 executed the
  evidence score of 11, malformed at
  the second call, and the risk seam
  at 10.5, unavailable at the third
  call; neither is the charged cell.
  The owner executed the charged cell
  in this repair pass, at the binary
  built from head cafe2822 (md5
  9a61b3763814c879e3728c3b94d5ae87),
  with a scripted judge answering a
  risk score of 11: three calls in
  order (choice, evidence, risk), the
  close lands unavailable, and the
  recorded reason is that the judge's
  risk answer is malformed. The six
  charged cells now stand discharged
  by execution, the risk cell by this
  item. Second, VL-85's sentence that
  the shell's merge risk note names
  the fix was true only of the shell's
  form at the build lane's read; the
  bot edited shell 6827098795 again at
  2026-11-12T15:07:42Z, and its form at
  this repair (8,777 chars, Merge Risk
  Low up to the closure head 7c8d5b7b,
  four passed checks, no Docstring
  row) carries no mention of VL-83 or
  the fix. The pull request body's
  step 0 figures are corrected by the
  owner's body edit to that form, and
  Round 12's read-back is charged with
  contracting this shell in its true
  final form on the closed record. The
  repair changes no other figure, no
  measurement, and no source.
- VL-87. Round 12 opens with the full
  read-back of pull request 15 (Round
  11), taken on 2026-11-12 through the
  API on its closed record, every review
  body to its end, every review comment
  thread to its end, and every issue
  comment. Reviews, one: review
  5876639927, COMMENTED by the review
  bot on the closure head 7c8d5b7b,
  submitted 2026-11-12T14:59:41Z,
  posting exactly one actionable
  comment. Review comments, one: the
  Minor 4552122144 against VL-83 at
  line 2317 of CONTRACT.md, that VL-83
  counted the judged pass as twelve
  judged closes where VL-82 records
  twelve contested opens. The finding
  was valid; it stands repaired in the
  repair commit cafe2822, with VL-85 as
  its record, and the comment's own
  thread carries no reply beyond the
  bot's later edit marking it addressed
  in that commit (the comment's updated
  stamp, 2026-11-12T15:03:45Z); the
  thread ends there. Issue comments,
  five: the bot's summary shell
  6827098795; the owner's trigger
  6827100537; the bot's action notice
  6827154011 (Action not completed,
  already reviewed the last commit,
  carrying no finding); the merging
  seat's landing comment 6828381681;
  and the parallel seat's concurrence
  6829940603. The census on the closed
  record is therefore one review, one
  review comment, and five issue
  comments, and the pull request body
  stands read in its final form as the
  owner's body edit left it (14,617
  chars, the repair head bdbbe58c and
  its tree be4d83db in the verified
  block, the owner repair note standing
  in it, and the bot's own release
  notes appendix at its tail, the
  bot's wording and no claim of the
  body). Nothing has arrived on the
  pull request after the concurrence at
  2026-11-12T16:48:24Z. The shell in
  its true final form at this read:
  created 2026-11-12T14:57:58Z and last
  edited 2026-11-12T15:47:37Z, 8,781
  chars, carrying a complete
  walkthrough of the change, Merge Risk
  Low up to the closure head 7c8d5b7b,
  and pre-merge checks at exactly four
  passed: Title, Linked Issues, Out of
  Scope Changes, and Description. Its
  change assessment commit and its
  merge risk coverage, source and
  covered alike, name the closure head
  7c8d5b7b4b6467e6e4186c3d376f585681edc8c3
  and no other head: neither repair
  head (cafe2822, bdbbe58c) is named
  anywhere in it. Its merge risk note
  records only that one summary line
  may misstate the number of judged
  closes, naming neither VL-83 nor the
  fix. No Docstring Coverage row stands
  anywhere in this form (the word
  occurs zero times), and no commits
  range section and no Run ID line
  stand in it. The shell's earlier
  generations stand on the record at
  their own moments: 11,475 chars as
  last edited 2026-11-12T15:03:37Z at
  the build lane's read; 8,777 chars at
  the edit of 2026-11-12T15:07:42Z, the
  form VL-86 records; 11,479 chars at
  the edit of 2026-11-12T15:36:26Z, the
  form the owner's body edit records;
  and the final form above at
  2026-11-12T15:47:37Z, at least four
  edits after its creation, so every
  shell figure in this round's items is
  bound to the moment of its read, the
  charge the landing comment left. The
  title's bound and the closure formula
  stand stated against the items' true
  range: the title bounds the round at
  VL-79 to VL-84, and VL-84 records the
  README line to 11 of 100 in the
  closure commit only, but the items'
  true range on the merged record is
  VL-79 to VL-86, VL-85 the step 0
  repair record and VL-86 the second
  repair record riding the two repair
  commits after the closure, so the
  title's bound names the slice to
  closure span and not the round's
  whole. The union of the seats' gate
  record grades as follows. The merging
  seat's three reviews of the repair
  head cafe2822 returned spec NOT PASS
  with two Minors, standards NOT
  APPROVE with one Minor, and red team
  APPROVE with no findings, its own
  corpus at 1,005 steps clean; the two
  Minors (VL-80's discharge list naming
  the evidence score of 11 where
  comment 6793880034 charged a risk
  score of 11, and the step 0
  disposition overtaken by the shell's
  later edits) stand repaired in VL-86
  at bdbbe58c, the charged risk cell
  executed by the owner in that repair.
  That seat's delta reviews of
  bdbbe58c returned one shared Minor,
  that VL-86 asserted the body
  correction before it landed; the
  owner's body edit then landed before
  the merge and discharges it exactly
  as the deltas specified. The parallel
  seat's delta reviews of the same head
  returned standards APPROVE with no
  findings and, on a second seating
  after its first spec delta lane
  wedged idle and was closed, spec PASS
  with no findings; the two sets differ
  in verdict on the same delta and the
  concurrence records that the body
  edit discharges the shared Minor
  under either reading. The merged
  state stands verified by both seats:
  merge 9c7e032a, its tree be4d83db
  equal to the final head's tree, the
  README on main at 11 of 100, and the
  contract on main running VL-79 to
  VL-86. The owed workspace copies of
  Round 11's scratch stand discharged
  as unrecoverable on this evidence:
  the scratch stood in /tmp/vl11 at
  landing (progress and the four
  instruments), /tmp has been wiped
  since, the directory does not stand
  on this box at this read, and no
  vl11-scratch copy stands in the
  workspace either; nothing is
  reconstructed, and Round 11's
  measurements stand on its contract
  record and its pull request record,
  not on the lost files. Late findings
  on pull request 15 stand disposed by
  this item: the only finding the bot
  ever posted there is 4552122144,
  disposed in Round 11 itself, and the
  shell's final form raises nothing
  beyond the risk note already named,
  which is that same point. No Round 11
  disposition is falsified by this
  record, nothing on the pull request
  stands undisposed, and no source or
  prose change is owed to it. Any late
  findings on this round's pull request
  are owed to Round 13's contract by
  the same shape.
- VL-88. The judgment battery stands
  re-run whole at this round's base
  against scripted judges only, the
  fake judge seated through the
  JEV_CALL transport and the fixture
  judgments file, twenty-five cells, no
  failures, and no live model called.
  The seam stands as contracted: the
  judge is consulted only on a
  contested close, its answers are
  taken in the order choice, evidence,
  risk, the model stands pinned at
  jev-1.13.0, and the confidence floor
  stands at 0.6 in src/judge.rs. The
  cells, each decided at its point.
  Where a close agrees with the
  suggestion the judge is never
  consulted, even with a failing judge
  configured (no calls, the recorded
  reason that the verdict and the
  suggestion agree). A contested close
  with no judge configured lands with
  the standing not consulted record (no
  judge is configured), no call made.
  At the floor itself: confidence of
  exactly 0.6 lands the close judged
  after exactly the three calls, the
  record carrying the judge's choice,
  the confidence standing as rendered,
  the two scores, and the pinned model;
  0.599999 refuses after the three
  calls, the registry byte-identical to
  its bytes before the close; 0.600001
  lands judged; the fifteen nines
  spelling of the shortfall refuses;
  the integer 1 lands judged and stands
  recorded as the integer; the integer
  0 refuses; and 6e-1 lands judged
  recording 0.6. An abstaining choice
  refuses at 0.99, at exactly 0.6, and
  at 0, the registry unchanged in each
  seat. The fixture seat replays the
  judged close with its registry
  byte-identical to the live seat's
  close of the same card, and refuses
  the abstaining replay with the
  registry unchanged. The unavailable
  family lands the close unavailable at
  its own call counts: a failed call at
  one call (the judge call failed); a
  non-JSON answer at one call (the
  judge's answer is not JSON); an
  unpinned model, jev-1.12.0, at one
  call (the judge's answer names an
  unpinned model); probabilities
  summing to one half at one call (the
  judge's choice answer is malformed);
  an evidence score of 11 at two calls
  (the judge's evidence answer is
  malformed); and a risk score of 11 at
  three calls (the judge's risk answer
  is malformed). The two score eleven
  cells stand as ordinary members of
  the battery from this round, the
  evidence cell contracted in VL-80 and
  the risk cell executed by the owner
  and recorded in VL-86. The score
  bounds land at the point itself:
  evidence 10 and risk 0 land judged,
  and a risk score of 10.5 turns the
  close unavailable at the third call.
  A close beside a rerun verdict naming
  a successor refuses at zero calls
  with the standing line (a successor
  is recorded only on a rerun verdict).
  The margins on the evidence score
  stand decided on the rendered diff: a
  margin of exactly 12.5 percent does
  not regress, and 13.28125 regresses
  and the close is refused. The band
  stands measured single-arm over the
  port's two transports, because the
  oracle carries no judge transport, as
  VL-80 claims.
- VL-89. The walk stands re-measured at
  the shipped binary on both arms, ten
  cells, no failures. The view's output
  path is spelled through a chain of
  directory links resolving onto the
  registry itself, the chain standing
  outside both work roots, and list and
  show are driven at the chain lengths
  where the law turns. On list, chains
  of 990 and 991 links refuse on both
  arms at exit 1 with the standing line
  (a view never overwrites the
  registry) and empty stdout; at 992
  the port goes out in silence at exit
  1 and the oracle goes out at exit 1
  under a traceback ending in
  RecursionError. On show the same
  points stand: 991 refuses by the
  standing line on both arms, and 992
  splits into the same crash class. The
  split therefore stands at exactly 991
  named refusal and 992 crash class on
  both surfaces, MAX_WALK_DEPTH 991 in
  src/cli.rs the port's own constant,
  probed at the shipped binary and
  inherited from no other crate. The
  flighttape pin stands re-confirmed
  live through the API in the same
  item: the pinned commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  carrying exactly fifty blobs, and the
  pinned crate's own suites at the
  cargo checkout run green, identity at
  thirteen tests and digest_cli at
  five, the golden fixture digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing pinned inside both suites
  and executed by them. The oracle's
  six Python files stand verified blob
  for blob against commit
  d62e5dc9547295f9ea7980f8298d905767b22f14
  through the API before any of this
  round's measurements: the commit's
  tree read through the API, its
  tarball fetched, and every one of its
  twenty-four blobs recomputed and
  matching, the six Python files among
  them. No walk or pin figure from the
  earlier rounds is falsified.
- VL-90. The realistic pass stands at
  1,270 steps against the oracle, above
  Round 11's 1,037 (126 opens, 801
  results, 105 closes, five refusal
  steps). A twin corpus of 140
  experiments is driven step for step
  at the port and at the oracle, the
  sources cycling eight fleet names,
  each experiment carrying two or three
  judged measures (seventy openings at
  two, seventy at three), the judged
  entries carrying their own margin on
  169 entries and the opening carrying
  an event level margin on thirty-five
  openings, and results between one and
  twelve per experiment with
  twenty-five histories running to
  twelve. Openings stand at 140,
  results at 1,006, closes at 118, and
  refusal steps at six, and 140 plus
  1,006 plus 118 plus six is the 1,270
  steps; no step's exit, stdout, or
  stderr differed between the twins.
  The closes divide at 72 in agreement
  with the recomputed suggestion and 46
  contested, and twenty-two experiments
  stand left open. The suggestion at
  every close stands recomputed in
  exact rational arithmetic under the
  VL-9 rule and checked against the
  recorded suggestion, with no
  disagreement. The six refusal steps
  stand refused identically on both
  arms: a show of a missing experiment;
  a result for an unopened experiment;
  a duplicate opening; a second close
  of a closed experiment; a keep close
  naming a successor against that
  closed experiment, refused on the
  already closed ground; and a keep
  close naming a successor against an
  open experiment, refused on the
  successor rule itself. The registry
  after the pass stands at 1,264 lines
  on each arm (140 opens, 1,006
  results, and 118 closes), 217,429
  bytes on the port and 207,335 bytes
  on the oracle, and the comparison
  stands made on the port's bytes after
  the judgment cut: 118 judgment spans
  cut on the port's registry, none on
  the oracle's, which carries no
  judgment member, each cut span
  verified to be exactly one of the two
  not consulted shapes before its
  removal, and the registry stands
  byte-identical after the cut. The
  final list stands identical after the
  same cut in its pretty form, 118
  spans cut; the final show stands
  identical for every one of the 140
  experiments after the same cut; and
  the final check stands identical on
  both arms and sound. The judged pass
  stands in its own counts over
  eighteen contested opens, every one
  resting on even measures under a keep
  suggestion and closed rerun under the
  scripted judge: six land judged
  (three at a confidence of 0.9 and
  three at exactly 0.6); six refuse
  with the registry byte-identical
  (three at 0.59 and three
  abstaining); and six land unavailable
  (three on a failed judge call and
  three on a malformed evidence score
  of 11). The judged pass registry
  stands at 48 lines and 17,034 bytes
  and reads sound under check. No live
  model was called anywhere in the
  pass.
- VL-91. The gate stands run at the
  final tree, whose source stands
  identical to the base tree's: cargo
  test passes at forty-seven tests
  (twenty-four judgment, twenty
  registry, and three library), clippy
  over all targets under denied
  warnings is clean, fmt check is
  clean, and the codepoint scan stands
  at zero over the tree and over all
  twenty-one blobs at the final head,
  U+2014 and U+2013 alike. Item three
  of the storefront standard stands
  re-verified through the API at this
  round's base: the description line
  stands byte-identical in the
  repository description, the
  Cargo.toml description field, and the
  README lede, the alignment VL-54
  contracted, and this round changes
  none of the three homes.
- VL-92. Round 12 closes. Its items
  stand at VL-87, the read-back of
  Round 11's pull request on its closed
  record with the four charges of its
  landing comment discharged in the
  item; VL-88, the judgment band
  re-measured at its exact points over
  twenty-five cells; VL-89, the walk at
  its points and the flighttape pin
  re-confirmed live; VL-90, the
  realistic pass at 1,270 steps with
  the judged pass in its own counts;
  and VL-91, the gate. The round's
  staging stands disclosed in its own
  record: a restart in the middle of
  the round wiped the lane's scratch
  whole, a corpus pilot then in flight
  with it, and the round's first three
  items stood already banked on the
  branch when it struck; the corpus of
  record stands measured whole after
  the restart on the rebuilt scratch,
  and no figure in these items is
  carried from the lost pilot. README
  records 12 of 100 only in the closure
  commit written after this round's
  three reviews, fixes, and owner
  verification.
- VL-93. Round 13 opens with the full
  read-back of pull request 16 (Round
  12), taken through the API on its
  closed record, the body read whole
  and every issue comment read whole.
  The census on the closed record: no
  reviews, no review comments, and four
  issue comments. The owner's trigger
  6845929400 (the review command, 20
  chars, 2026-11-13T06:28:20Z). The
  bot's summary shell 6845929973 stands
  contracted in its true final form at
  this read: created
  2026-11-13T06:28:22Z and last edited
  2026-11-13T06:32:21Z, 11,279 chars,
  carrying a complete walkthrough of
  the change, the statement that no
  actionable comments were generated in
  the recent review, Merge Risk Minimal
  up to the closure head d4fd958e with
  its merge risk coverage naming that
  head as both source and covered, its
  change assessment commit the same
  head, pre-merge checks at exactly
  four passed (Description, Title,
  Linked Issues, and Out of Scope
  Changes), no Docstring Coverage row
  (the word occurs zero times), and Run
  ID
  528a5156-feaf-43b5-b1d8-102f7e69bc3f.
  The pull request body's own step 0
  section stated the shell at 11,277
  chars with a last edit of
  2026-11-13T06:30:25Z; the shell
  drifted two characters after that
  read to the form contracted here, and
  no disposition fact moved with the
  drift. The completed review raised
  nothing, so nothing stood to fix and
  nothing was dismissed. The bot's
  auto-reply 6845985341 (573 chars,
  2026-11-13T06:31:42Z) is an action
  notice stating the last commit was
  already reviewed; it carries no
  finding, and it arrived after the
  body's census, which named two issue
  comments at its read. The merging
  seat's landing comment 6846459678
  (1,647 chars, 2026-11-13T07:00:29Z)
  closes the record, and nothing has
  arrived on the pull request after it:
  no late CodeRabbit finding stands on
  pull request 16, and the landing's
  third charge discharges with nothing
  owed. The body's release notes
  appendix stands in it with two
  bullets, each correct at its scope.
  The merge stands as the landing
  recorded it: merge commit
  1ab2e17d75203d89ee577b9c01ee7d25c9e55026,
  its tree
  55adaf282fdcbe78de0ca5b7f2b1169f722c4412
  equal to the reviewed head
  d4fd958e's, and README at 12 of 100
  on main.
- VL-94. The judgment battery stands
  re-run whole at this round's base
  against scripted judges only, the
  fake judge seated through the
  JEV_CALL transport and the fixture
  judgments file, twenty-five cells, no
  failures, and no live model called.
  The seam stands as contracted in
  VL-88: the judge is consulted only on
  a contested close, its answers taken
  in the order choice, evidence, risk,
  the model pinned at jev-1.13.0, and
  the confidence floor at 0.6 in
  src/judge.rs; the cells stand decided
  at the same points VL-88 enumerates.
  The round's first clean run stood at
  nineteen of twenty-five: the six
  malformed-family cells (the unpinned
  model, the halved probabilities, the
  evidence score of 11, the risk score
  of 11, the score bounds, and the risk
  score of 10.5) failed on a defect in
  this round's own fake judge, whose
  choice branch raised KeyError for
  those six modes, so the port recorded
  the judge call failed at the first
  call. The fake was repaired to
  default those modes to a confidence
  of 0.9, and the re-run of record
  stands at twenty-five of twenty-five,
  the six cells at their contracted
  dispositions: the unpinned model and
  each malformed answer land
  unavailable with their stated
  reasons, and the bounds cell lands
  judged at an evidence strength of 10
  and a regression risk of 0.
- VL-95. The walk stands re-measured at
  the shipped binary on both arms, ten
  cells, no failures at this lane's
  re-run: list at 990, 991, and 992
  links and show at 991 and 992, the
  split standing at exactly 991 named
  refusal and 992 crash class on both
  surfaces, MAX_WALK_DEPTH 991 in
  src/cli.rs the port's own constant.
  The flighttape pin stands
  re-confirmed live through the API in
  the same item: the pinned commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  carrying exactly fifty blobs, and the
  pinned crate's own suites at the
  cargo checkout run green at nineteen
  suites and 241 tests with none
  failed, identity at thirteen tests
  and digest_cli at five, the golden
  fixture digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing pinned inside both suites
  and executed by them. The oracle's
  six Python files stand verified blob
  for blob against commit
  d62e5dc9547295f9ea7980f8298d905767b22f14
  before any of this round's
  measurements: the commit's tree read
  through the API and every one of its
  twenty-four blobs recomputed and
  matching. One correction to the
  round's own brief stands recorded
  here: the brief named a second pin in
  this repo (onrecord at 9e41a7be); no
  such citation stands anywhere in this
  repo, the contract, Cargo.toml,
  Cargo.lock, README, and history all
  searched, and the repo's only pin is
  the flighttape pin above. No walk or
  pin figure from the earlier rounds is
  falsified.
- VL-96. The realistic pass stands
  re-measured at the larger scale this
  round's scale correction set: a twin
  corpus of one hundred eighty
  experiments driven step for step at
  the port and at the oracle, the
  generator seeded 20261116. The
  generator as first seeded built one
  hundred sixty experiments totalling
  1,178 steps, below Round 12's 1,270;
  the experiment count stands raised to
  one hundred eighty with the seed and
  every other construction rule
  unchanged, and the pass as executed
  totals 1,346 steps: 180 opens, 1,024
  results, 136 closes (90 in agreement,
  46 contested), and 6 refusal steps,
  with 44 experiments left open. The
  twin pass stands executed twice to
  completion; the two executions'
  registries are byte-identical on both
  arms, 1,340 lines per arm (232,193
  bytes at the oracle, 243,925 at the
  port), the port registry carrying 136
  judgment spans the oracle predates,
  one per close, and standing equal to
  the oracle registry byte for byte
  once those spans are cut. The
  suggestion and the regressed list
  recompute exactly under the VL-9 rule
  at all 272 closes, both registries
  counted. The closing phases stand
  re-run against the surviving
  registries and banked: the six
  refusal steps land identically on
  both arms, the final check and list
  agree, a show of every one of the 180
  experiments agrees across the arms,
  and a judged pass over eighteen
  contested opens closes the
  measurement on the port side only,
  six judged, six refused, and six
  unavailable, its registry standing at
  48 lines and 14,643 bytes with the
  closing check sound. A third
  execution under a hardened driver
  banked its verdict as it ran: the box
  write fault refused one arm's
  registry write at four steps (a
  result at exp-r13-0033 on the oracle
  side; a result at exp-r13-0034 and a
  result and a close at exp-r13-0035 on
  the port side), the run's hollow-pass
  guard closed it at 1,337 port lines
  for 1,340 landed steps, and it stands
  counted as fault evidence, not as a
  measurement; the two completed
  executions are the run of record. No
  live model is called anywhere in the
  pass.
- VL-97. The gate stands run at the
  base tree and at the final tree,
  whose source stands identical to the
  base tree's: cargo test passes at
  forty-seven tests (twenty-four
  judgment, twenty registry, and three
  library), clippy over all targets
  under denied warnings is clean, fmt
  check is clean, and the codepoint
  scan stands at zero over the tree and
  over all twenty-one blobs at the
  final head, U+2014 and U+2013 alike.
  Item three of the storefront standard
  stands re-verified through the API at
  this round's base: the description
  line stands byte-identical in the
  repository description, the
  Cargo.toml description field, and the
  README lede, the alignment VL-54
  contracted, and this round changes
  none of the three homes.
- VL-98. Round 13 closes. Its items
  stand at VL-93, the read-back of
  Round 12's own pull request; VL-94,
  the judgment battery re-run whole at
  twenty-five of twenty-five; VL-95,
  the walk re-measured, the flighttape
  pin re-confirmed live, and the oracle
  verified blob for blob; VL-96, the
  realistic pass at the reworked scale
  of 1,346 steps; and VL-97, the gate
  and the storefront's third item. The
  harness provenance this round's
  closure was charged to carry stands
  here in its own text: the
  measurements ran on four drivers kept
  outside the repo (battery13.py,
  walk13.py, corpus13.py, and
  fake_judge13.py, the scripted judge
  the battery and the judged pass
  consult through the JEV_CALL
  transport), the corpus driver
  descending from the Round 9 battery's
  corpus section and its generator
  seeded 20261116 with the experiment
  count raised from one hundred sixty
  to one hundred eighty as VL-96
  records. Two defects in this round's
  own drivers stand recorded as driver
  defects, not crate defects: the fake
  judge's choice branch raised KeyError
  for the six malformed-family modes,
  repaired by defaulting those modes to
  a confidence of 0.9 before the
  battery's run of record; and the
  corpus driver's first completed
  executions banked nothing when the
  box write fault returned at their
  closing phases, answered by a
  hardened copy (corpus13b.py,
  differing only in banking) whose run
  banked the fault's own four refused
  writes, and by a tail driver
  (corpus13_tail.py) that re-ran the
  closing phases against the surviving
  registries. The round was built
  across three lanes in succession,
  each standing down clean on an
  ownership transfer with its records
  banked in the line's scratch, and the
  corpus's final execution ran without
  a build-slot hold because the driver
  invokes no cargo work, the slot
  standing frozen behind another lane's
  probe gate; both facts stand
  disclosed here and in the pull
  request. The README records 13 of 100
  in the closure commit written after
  this round's three reviews, fixes,
  and owner verification.
- VL-99. The read-back of Round 13's
  pull request stands contracted here
  on its closed record, read whole
  through the API at this round's hand.
  The pull request stands closed and
  merged: its body stands at 16,447
  characters with the owner completion
  record at its tail, its head
  1ff17e623a24e145abcfc7dfa62e0dd82b71189a
  over the Round 12 merge
  1ab2e17d75203d89ee577b9c01ee7d25c9e55026,
  and it landed through the merge queue
  as squash commit
  ac93ef91ba356d36c8bd9989fda6ef121ab3792a,
  single parent the Round 12 merge, the
  landing tree
  d4877275072a5f5a195ddcd483dc22e63029fa1e
  verified by the owner blob for blob
  against the reviewed tree, twenty-one
  blobs, and the README read on main at
  13 of 100. The census at this read
  stands at six issue comments, one
  review, and one review comment: the
  trunk note 6951787931 in its landed
  form (113 characters, merged
  successfully, last edited
  2026-11-17T16:04:37Z), the CodeRabbit
  shell 6951793767 in its true final
  form at this read (8,723 characters,
  created 2026-11-17T15:01:35Z, last
  edited 2026-11-17T15:56:06Z, Merge
  Risk Low, coverage source and covered
  both at the final head, four
  pre-merge checks passed, no Docstring
  Coverage row), the trigger 6951837635
  (20 characters), the bot auto-reply
  6951890477 (573 characters, action
  not completed, already reviewed the
  last commit, no finding), the merge
  comment 6953278379 (12 characters),
  and the landing comment 6953548156
  (1,582 characters) carrying this
  round's charges; the landing census
  counted five issue comments before
  the landing comment itself posted,
  and nothing stands after it. The
  shell's drift stands recorded in
  three forms past the build lane's
  census: 8,719 characters as the
  body's step 0 section stated, 8,721
  at the owner's hand (last edited
  2026-11-17T15:09:49Z), and 8,723 at
  the landing census and unchanged at
  this read, no disposition fact moved
  at any step. The one review,
  5940955405 (COMMENTED, Run ID
  48f64524-9e17-4234-9224-5b2b3d0904e9,
  profile CHILL), carries exactly one
  review comment, 4601434050 on this
  contract (Minor, functional
  correctness, the VL-97 gate
  sentence), dispositioned in the body
  as valid as raised against the body's
  placeholder state and cured there by
  citing the hosted runs; no late
  CodeRabbit finding stands on the pull
  request. The two corrections the
  reviews left stand contracted here.
  First, VL-97's local final-tree gate
  never ran and never will: its waiter
  ended with the build lane, its status
  file stands at its waiting line
  (waiting for build slot
  2026-11-17T14:03:22Z), its console
  log stands at zero bytes, and no
  final-tree gate log with content
  exists; the final tree's gate of
  record is the hosted one, runs
  44108904949 and 44109547394 on the
  exact head. Second, the third corpus
  execution's fourth fault step, the
  result at exp-r13-0033, was an
  exit-class divergence on the oracle
  arm whose registry line verifiably
  landed, not a refused registry write;
  three of the four fault steps were
  refused registry writes, and the
  count of four fault steps stands. One
  item stands carried from Round 13's
  reviews by name: the red team's
  battery slice, the six
  malformed-family cells and two
  ordinary cells staged with
  byte-identical instruments against
  the chain binary, never executed, the
  box write fault refusing every setup
  write at its hand and its banked
  results file standing at zero bytes;
  VL-100 of this round discharges it by
  execution at this round's hand.
- VL-100. The judgment battery stands
  re-run whole at this round's base
  against scripted judges only, the
  fake judge seated through the
  JEV_CALL transport and the fixture
  judgments file, twenty-five cells, no
  failures, and no live model called.
  The seam stands as contracted in
  VL-88: the judge is consulted only on
  a contested close, its answers taken
  in the order choice, evidence, risk,
  the model pinned at jev-1.13.0, and
  the confidence floor at 0.6 in
  src/judge.rs; the cells stand decided
  at the same points VL-88 enumerates.
  The fake judge stands as Round 13
  repaired it, its choice branch
  defaulting the six malformed-family
  modes to a confidence of 0.9
  (CONF_LITERAL.get), the instrument
  copied byte for byte from Round 13's
  (md5
  a79ded49ae7285fd2d7b028c8b47dd7f) and
  the battery driver renamed only in
  its round furniture. The carried item
  from Round 13's reviews stands
  discharged by this execution: the six
  malformed-family cells the red team's
  slice could not execute ran at this
  round's hand on the port arm as the
  battery is constructed, C17 the
  unpinned model unavailable with the
  reason the judge's answer names an
  unpinned model after the choice call
  alone, C18 the halved probabilities
  unavailable, the judge's choice
  answer is malformed, C19 the evidence
  score of 11 unavailable, the judge's
  evidence answer is malformed, C20 the
  risk score of 11 unavailable, the
  judge's risk answer is malformed, C21
  the score bounds judged at an
  evidence strength of 10 and a
  regression risk of 0, and C22 the
  risk score of 10.5 unavailable, the
  judge's risk answer is malformed; the
  two ordinary cells of that slice, C03
  and C04, ran with them at their
  contracted dispositions. The run of
  record stands at twenty-five of
  twenty-five.
- VL-101. The walk stands re-measured
  at the shipped binary on both arms,
  ten cells, no failures: list at 990,
  991, and 992 links and show at 991
  and 992, the split standing at
  exactly 991 named refusal and 992
  crash class on both surfaces,
  MAX_WALK_DEPTH 991 in src/cli.rs. The
  flighttape pin stands re-confirmed
  live through the API: the pinned
  commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  carrying exactly fifty blobs, and the
  pinned crate's own suites at the
  cargo checkout run green at nineteen
  suites and 241 tests with none
  failed, identity at thirteen tests
  and digest_cli at five, the golden
  fixture digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing pinned inside both suites
  and executed by them. The oracle's
  six Python files stand verified blob
  for blob against commit
  d62e5dc9547295f9ea7980f8298d905767b22f14
  before any of this round's
  measurements: the commit's tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  read through the API, its twenty-four
  blobs matching Round 13's saved
  census, and every one of the
  twenty-four recomputed from the local
  extraction and matching. The repo's
  only pin is the flighttape pin, as
  VL-95 corrected the record.
- VL-102. The corrupted-registry reads
  stand measured fresh, a surface no
  earlier round measured: one healthy
  registry built by identical twin
  steps on both arms (six opens, six
  results, four closes, sixteen lines,
  2,962 bytes in the oracle arm's
  bytes, the port's registry equal
  after the prescribed cut at every
  build step, no judge configured), and
  the read surfaces, check, list, and
  show, driven on both arms over the
  healthy bytes and twelve mutated
  copies (truncation mid-line, the
  final newline removed, one digit
  changed in a result's measures, the
  JSON syntax broken on a middle line,
  a line duplicated, two lines swapped,
  a blank line inserted, an unknown
  event kind, the empty file, a
  required field removed, CRLF line
  endings, a trailing blank line):
  thirteen cells, thirty-nine twin
  comparisons, thirty-nine in
  agreement, no cell failed and none
  blocked, the registry bytes unchanged
  by every read. The cut the build and
  the show surface rely on is the VL-96
  corpus discipline: the port's closes
  add a not_consulted judgment member
  the oracle predates, brace-matched,
  parsed, and verified to be exactly
  one of its two prescribed shapes
  before removal; a span of any other
  shape would have convicted. The
  driver's first build attempt stopped
  at the first close on that asymmetry
  and the driver took up the cut before
  any cell ran, an amendment recorded
  in the lane's progress record before
  the run.
- VL-103. The gate stands run at the
  base tree and at the final tree,
  whose source stands identical to the
  base tree's: cargo test passes at
  forty-seven tests (twenty-four
  judgment, twenty registry, and three
  library), clippy over all targets
  under denied warnings is clean, fmt
  check is clean, and the codepoint
  scan stands at zero over the tree and
  over all twenty-one blobs at the
  final head, U+2014 and U+2013 alike,
  the scan proven on a planted control.
  The base extraction stands verified
  blob for blob against the landing
  tree of Round 13's merge (twenty-one
  blobs recomputed and matching) with
  main read live at
  ac93ef91ba356d36c8bd9989fda6ef121ab3792a.
  Item three of the storefront standard
  stands re-verified through the API at
  this round's base: the description
  line stands byte-identical in the
  repository description, the
  Cargo.toml description field, and the
  README lede, and this round changes
  none of the three homes; the item
  stands passing, standing, not
  repaired.
- VL-104. Round 14 closes. Its items
  stand at VL-99, the read-back of
  Round 13's own pull request with the
  two corrections contracted and the
  carried battery slice named; VL-100,
  the judgment battery re-run whole at
  twenty-five of twenty-five,
  discharging the carried slice by
  execution; VL-101, the walk
  re-measured, the flighttape pin
  re-confirmed live, and the oracle
  verified blob for blob; VL-102, the
  corrupted-registry reads at
  thirty-nine of thirty-nine
  comparisons; and VL-103, the gate and
  the storefront's third item standing.
  The harness provenance stands here in
  its own text: the measurements ran on
  drivers kept outside the repo
  (battery14.py and fake_judge14.py,
  the battery driver renamed only in
  its round furniture from Round 13's
  and the fake judge copied byte for
  byte, walk14.py likewise renamed,
  corrupt14.py new this round, and
  verify_census14.py for the pin,
  oracle, and storefront reads). The
  measurement runs executed in the
  lane's staging against Round 13's
  chain binary (sha256
  4e5ee9dca8bcc591786db8576f11009e1b55a6a15015829f9a21cd1b4a018517,
  built from the base checkout whose
  every source blob stands identical to
  this round's base) without a
  build-slot hold, because the drivers
  invoke no cargo work, on the VL-96
  and VL-98 precedent; the round's
  first cargo chain died queued in a
  box restart, its successor held the
  slot through a long closed wave of
  the fault and died in a second
  restart with its probe gate never
  opened, and a third launch in the
  post-restart window ran every gate
  stage green under the slot; these
  facts stand disclosed here and in the
  pull request. The README records 14
  of 100 in the closure commit written
  after this round's three reviews,
  fixes, and owner verification.
- VL-105. The read-back of Round 14's
  landing pull request stands
  contracted here on its closed record,
  read whole through the API at this
  round's hand, with its origin pull
  request's record read with it. Pull
  request 20 stands closed and merged:
  its body stands at 12,440 characters,
  opening with the owner's restack note
  and the owner completion record and
  carrying pull request 19's body
  unchanged below them, its head
  66f6537f7fc9a1a340f5d994338dd783ad5a9cd4
  over the base
  ac93ef91ba356d36c8bd9989fda6ef121ab3792a,
  and it landed through the merge queue
  as squash commit
  073c114a02a214d8bb2df1d051c2b299a45e5135,
  single parent
  ac93ef91ba356d36c8bd9989fda6ef121ab3792a,
  the landing tree
  610071b9ba9a80d3fb81455d82cfe92532bac786
  verified by the owner as the reviewed
  tree itself and re-read at this
  round's hand (the landing commit's
  tree stands at 610071b9 on a fresh
  read), and the README read on main at
  14 of 100. The census at this read
  stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 6962477743 in its landed
  form (113 characters, merged
  successfully, created
  2026-11-17T22:11:54Z, last edited
  2026-11-17T22:19:35Z), the CodeRabbit
  shell 6962482631 in its true final
  form at this read (11,325 characters,
  created 2026-11-17T22:12:07Z, last
  edited 2026-11-17T22:14:36Z and
  unchanged since, Run ID
  767361e2-d1bb-443f-bb50-3a444ef73bbe,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head 66f6537f,
  four pre-merge checks passed, and its
  recent-review section stating that no
  actionable comments were generated),
  the owner's merge comment 6962493336
  (12 characters,
  2026-11-17T22:12:36Z), and the
  landing comment 6962723776 (1,857
  characters, 2026-11-17T22:22:27Z)
  carrying this round's charges; the
  owner's landing census counted three
  issue comments before the landing
  comment itself posted, the figure at
  this read agrees with the owner's
  shell figure exactly, and nothing
  stands after the landing comment.
  Pull request 19, the round's origin,
  stands closed and unmerged, its body
  at 8,018 characters, its head
  baa6acb5e49b81438dc37016fbb0fa42d079301b
  over the same base, its census at
  four issue comments, no reviews, and
  no review comments: the trunk note
  6960274048 in its final form (199
  characters, the pull request removed
  from the merge queue because it was
  closed, last edited
  2026-11-17T22:12:30Z), the CodeRabbit
  shell 6960279054 in its true final
  form at this read (11,069 characters,
  created 2026-11-17T20:36:17Z, last
  edited 2026-11-17T20:55:29Z and
  unchanged since, Run ID
  0a54b547-0a3a-40f7-b14f-e2b1e1e93944,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head baa6acb5,
  no actionable comments generated),
  the merge comment 6960752486 (12
  characters), and the owner's pointer
  closure 6962480359 (619 characters,
  2026-11-17T22:12:01Z) recording that
  the pull request could not land as
  branched, its mergeability computed
  against the stale Round 12 merge base
  and the queue's verdict notMergeable
  on that ground alone, closed in
  favour of pull request 20 carrying
  the identical reviewed tree. The
  owner completion record on pull
  request 20 stands read with it: the
  three reviews each APPROVE (spec
  fidelity with one Minor, the step 0
  figure wording on pull request 19;
  standards with one Minor, the slice
  staging file overwritten by the
  closure staging on disk, no figure or
  claim affected, this round charged
  with keeping slice and closure
  staging in separate files; the red
  team with no findings, the battery
  and the corrupted-registry surface
  reproduced whole at its own hand with
  results byte-identical to the build
  lane's), and storefront item 3
  standing reported as re-verified,
  standing, not repaired.
- VL-106. The standing regression
  stands re-run whole in its contracted
  forms. The judgment battery stands
  re-run at this round's base against
  scripted judges only, the fake judge
  seated through the JEV_CALL
  transport, twenty-five cells, no
  failures, and no live model called;
  the fake judge stands as Round 13
  repaired it, copied byte for byte
  (md5
  a79ded49ae7285fd2d7b028c8b47dd7f),
  and the battery driver renamed only
  in its round furniture from Round
  14's. The six malformed-family cells
  stand at exactly the dispositions
  VL-100 contracted: C17 the unpinned
  model unavailable with the reason the
  judge's answer names an unpinned
  model after the choice call alone,
  C18 the halved probabilities
  unavailable, the judge's choice
  answer is malformed, C19 the evidence
  score of 11 unavailable, the judge's
  evidence answer is malformed, C20 the
  risk score of 11 unavailable, the
  judge's risk answer is malformed, C21
  the score bounds judged at an
  evidence strength of 10 and a
  regression risk of 0, and C22 the
  risk score of 10.5 unavailable, the
  judge's risk answer is malformed; the
  run of record stands at twenty-five
  of twenty-five. The walk stands
  re-measured at the shipped binary on
  both arms, ten cells, no failures:
  list at 990, 991, and 992 links and
  show at 991 and 992, the split
  standing at exactly 991 named refusal
  and 992 crash class on both surfaces,
  MAX_WALK_DEPTH 991 in src/cli.rs. The
  flighttape pin stands re-confirmed
  live through the API: the pinned
  commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  carrying exactly fifty blobs, and the
  pinned crate's own suites at the
  cargo checkout run green at nineteen
  suites and 241 tests with none
  failed, identity at thirteen tests
  and digest_cli at five, the golden
  fixture digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing pinned inside both suites
  and executed by them. The oracle's
  files stand verified against commit
  d62e5dc9547295f9ea7980f8298d905767b22f14:
  the commit's tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  read through the API, its twenty-four
  blobs matching Round 13's saved
  census, and every one of the
  twenty-four recomputed from the local
  extraction and matching. The repo's
  only pin is the flighttape pin.
- VL-107. The gate stands run at the
  base tree and at the head tree, both
  under the build slot. The base tree
  was extracted and blob-verified
  firsthand before any gate ran on it:
  the clone working tree recomputes to
  main's tree 610071b9 over all
  twenty-one blobs with zero
  mismatches, verified at this lane's
  hand at lane start and again by the
  chain's verify stage, exit 0. The
  base gate at the chain's hand: cargo
  build exit 0; cargo test exit 0 with
  forty-seven passed and none failed
  (three library, twenty-four judgment,
  twenty in the registry suite); cargo
  clippy across all targets with
  warnings denied, exit 0; cargo fmt
  --check exit 0; the repo codepoint
  scan over the base tree at zero em
  dashes and zero en dashes, exit 0,
  the planted control detected at one
  of each. The gate at the final tree,
  whose source stands identical to the
  base tree's, stands run under the
  slot at the continuation lane's hand:
  cargo test exit 0 with forty-seven
  passed and none failed in the same
  split, cargo clippy across all
  targets with warnings denied exit 0,
  cargo fmt --check exit 0, and the
  repo codepoint scan at zero em dashes
  and zero en dashes over the final
  tree, exit 0.
- VL-108. The absent registry and the
  unknown experiment stand measured
  fresh, a surface no earlier round
  measured: twin fresh directories on
  both arms, each starting with no
  registry file at all, and twelve
  cells in one fixed sequence, the read
  surfaces (check, list, show) driven
  on the absent registry, the write
  surfaces (open, result) creating the
  registry from absence, show on the
  opened experiment, the unknown
  experiment exp-zz-9999 driven at
  show, result, and close on the
  healthy registry, a close of the
  opened experiment with no judge
  configured, and the read surfaces
  again on the closed registry. The
  amended run of record agrees on
  twelve of twelve cells, none failed,
  none blocked: the three absence cells
  return the clean refusal at exit 1
  with the registry still absent on
  both arms, the unknown-experiment
  cells refuse at exit 1, and the
  post-close reads agree after the cut.
  No conviction; the surface joins the
  standing regression's evidence. The
  registry after the result cell holds
  exactly two lines and after the close
  exactly three, on both arms. The cut
  the close cell and the post-close
  read cells rely on is the VL-96
  corpus discipline: the port's closes
  add a not_consulted judgment member
  the oracle predates, standing in the
  closed registry's bytes and in the
  port's list rendering of it,
  brace-matched, parsed, and verified
  to be exactly one of its two
  prescribed shapes before removal; a
  span of any other shape would have
  convicted. The driver's first run
  banked ten of twelve with the two
  post-close read cells failing on
  exactly that span, a derivation miss
  in this round's cell definitions,
  which named the cut only for the
  close cell; the amendment extending
  the cut to the post-close read cells
  stands recorded in the lane's
  progress record before the re-run,
  and the amended run is the run of
  record.
- VL-109. Item three of the storefront
  standard stands re-verified through
  the API at this round's base: the
  description line stands
  byte-identical in the repository
  description, the Cargo.toml
  description field, and the README
  lede, all three read at this round's
  hand, and this round changes none of
  the three homes; the item stands
  passing, standing, not repaired.
- VL-110. Round 15 closes. Its items
  stand at VL-105, the read-back of
  Round 14's landing pull request with
  its origin pull request read with it,
  the shell in its true final form at
  the reading hand; VL-106, the
  standing regression re-run whole, the
  battery at twenty-five of
  twenty-five, the walk at ten of ten,
  the flighttape pin re-confirmed live
  with its suites green, and the oracle
  verified blob for blob; VL-107, the
  gate at the base tree and at the
  final tree; VL-108, the absent
  registry and the unknown experiment
  at twelve of twelve cells agreeing,
  no conviction; and VL-109, the
  storefront's third item standing,
  re-verified, not repaired. The
  harness provenance stands here in its
  own text: the measurements ran on
  drivers kept outside the repo
  (battery15.py and fake_judge15.py,
  the battery driver renamed only in
  its round furniture from Round 14's
  and the fake judge copied byte for
  byte, walk15.py likewise renamed,
  absent15.py new this round, and
  verify_census15.py for the pin,
  oracle, and storefront reads). The
  measurement runs executed in the
  lane's staging against Round 13's
  chain binary (sha256
  4e5ee9dca8bcc591786db8576f11009e1b55a6a15015829f9a21cd1b4a018517,
  built from a checkout whose every
  source blob stands identical to this
  round's base) without a build-slot
  hold, because the drivers invoke no
  cargo work, on the VL-96 and VL-98
  precedent. The round's cargo work ran
  under the build slot: the base gate
  green in the chain's third launch
  after two restart kills in the queue,
  the pin suite at the continuation
  lane's hand after the chain's pin
  stage died on the box's file-count
  quota and the detached re-run's probe
  gate never opened in the closed wave
  that followed, and the final-tree
  gate at the continuation lane's hand.
  The round's first build lane errored
  out on 2026-11-18 with its record
  banked through the base gate, and a
  continuation lane adopted the banked
  record whole and finished the round;
  every figure in these items traces to
  a banked run in the lane's record.
  The absent driver's first run banked
  ten of twelve on a derivation miss in
  its own cell definitions, the
  amendment recorded in the progress
  record before the re-run, and the
  amended run is the run of record.
  Slice and closure staging stand in
  separate directories this round, as
  Round 14's standards Minor charged,
  so both halves of the pushed-tree
  assertion stay re-performable from
  disk. The README records 15 of 100 in
  the closure commit written after this
  round's three reviews, fixes, and
  owner verification.
- VL-111. The read-back of Round 15's
  landing pull request stands
  contracted here on its closed record,
  read whole through the API at this
  round's hand. Pull request 22 stands
  closed and merged: its body stands at
  12,187 characters, the lane's body
  with the owner completion record
  appended after the CodeRabbit
  appendix, its head
  45eb10ec516fdbe0f3ab908aa73c1a9fab4783a8
  over the base
  073c114a02a214d8bb2df1d051c2b299a45e5135,
  and it landed through the merge queue
  as squash commit
  9a5af9eb6ba2a23b6efabc547eb507086d278a7c,
  single parent
  073c114a02a214d8bb2df1d051c2b299a45e5135,
  the landing tree
  c210f6b9d78573b8f701edbe9a6cd5db2d874060
  verified by the owner as the reviewed
  tree itself and re-read at this
  round's hand (main stands at 9a5af9eb
  on a fresh read, its tree at c210f6b9
  over twenty-one blobs), and the
  README read on main at 15 of 100. The
  census at this read stands at four
  issue comments, one review, and two
  review comments: the trunk note
  7025706559 in its landed form (113
  characters, merged successfully,
  created 2026-11-19T23:59:45Z, last
  edited 2026-11-20T00:17:50Z), the
  CodeRabbit shell 7025709971 in its
  true final form at this read (8,681
  characters, created
  2026-11-19T23:59:56Z, last edited
  2026-11-20T00:10:43Z, Run ID
  49c9dc1a-fc78-4f84-b2da-7938464b4ace
  read from its review object, profile
  CHILL, Merge Risk Low, coverage at
  the exact head 45eb10ec, four
  pre-merge checks passed), the merge
  comment 7025929296 (12 characters,
  2026-11-20T00:10:42Z), and the
  landing comment 7027860845 (1,610
  characters, 2026-11-20T01:50:58Z)
  carrying this round's charges; the
  review 5985446528 stands COMMENTED
  (4,059 characters) with its two
  inline Minors (4636800529 on
  CONTRACT.md and 4636800535 on
  README.md line 135), both dismissed
  in the body with reasons; the owner's
  completion record stands read with
  it: the three reviews each APPROVE
  with no findings at any grade, the
  battery and the absent-registry
  surface reproduced byte-identically
  at the red team's hand, both step 0
  dismissals sustained on their merits
  by the standards review, and
  storefront item 3 standing reported
  as re-verified, standing, not
  repaired. Nothing stands after the
  landing comment.
- VL-112. The standing regression
  stands re-run whole in its contracted
  forms. The judgment battery stands
  re-run at this round's base against
  scripted judges only, the fake judge
  seated through the JEV_CALL
  transport, twenty-five cells, no
  failures, and no live model called;
  the fake judge stands as Round 13
  repaired it, copied byte for byte
  (md5
  a79ded49ae7285fd2d7b028c8b47dd7f),
  and the battery driver renamed only
  in its round furniture from Round
  15's. The six malformed-family cells
  stand at exactly the dispositions
  VL-100 contracted: C17 the unpinned
  model unavailable with the reason the
  judge's answer names an unpinned
  model after the choice call alone,
  C18 the halved probabilities
  unavailable, the judge's choice
  answer is malformed, C19 the evidence
  score of 11 unavailable, the judge's
  evidence answer is malformed, C20 the
  risk score of 11 unavailable, the
  judge's risk answer is malformed, C21
  the score bounds judged at an
  evidence strength of 10 and a
  regression risk of 0, and C22 the
  risk score of 10.5 unavailable, the
  judge's risk answer is malformed; the
  run of record stands at twenty-five
  of twenty-five. The walk stands
  re-measured at the shipped binary on
  both arms, ten cells, no failures:
  list at 990, 991, and 992 links and
  show at 991 and 992, the split
  standing at exactly 991 named refusal
  and 992 crash class on both surfaces,
  MAX_WALK_DEPTH 991 in src/cli.rs. The
  flighttape pin stands re-confirmed
  live through the API: the pinned
  commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  carrying exactly fifty blobs, and the
  pinned crate's own suites at the
  cargo checkout run green at nineteen
  suites and 241 tests with none
  failed, identity at thirteen tests
  and digest_cli at five, the golden
  fixture digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing pinned inside both suites
  and executed by them. The oracle's
  files stand verified against commit
  d62e5dc9547295f9ea7980f8298d905767b22f14:
  the commit's tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  read through the API, its twenty-four
  blobs matching Round 13's saved
  census, and every one of the
  twenty-four recomputed from the local
  extraction and matching. The repo's
  only pin is the flighttape pin.
- VL-113. The gate stands run at the
  base tree and at the head tree, both
  under the build slot. The base tree
  was extracted and blob-verified
  firsthand before any gate ran on it:
  the base tree extracted blob by blob
  through the API recomputes to main's
  tree c210f6b9 over all twenty-one
  blobs with zero mismatches, verified
  at this lane's hand at lane start and
  again by the chain's verify stage,
  exit 0. The base gate at the chain's
  hand: cargo build exit 0; cargo test
  exit 0 with forty-seven passed and
  none failed (three library,
  twenty-four judgment, twenty in the
  registry suite); cargo clippy across
  all targets with warnings denied,
  exit 0; cargo fmt --check exit 0; the
  repo codepoint scan over the base
  tree at zero em dashes and zero en
  dashes, exit 0, the planted control
  detected at one of each. The gate at
  the final tree, whose source stands
  identical to the base tree's, stands
  run under the slot at this lane's
  hand: cargo test exit 0 with
  forty-seven passed and none failed in
  the same split, cargo clippy across
  all targets with warnings denied exit
  0, cargo fmt --check exit 0, and the
  repo codepoint scan at zero em dashes
  and zero en dashes over the final
  tree, exit 0.
- VL-114. The rerun successor stands
  measured fresh, a surface no earlier
  round measured: twin fresh
  directories on both arms, and
  fourteen cells in one fixed sequence,
  an experiment opened and given a
  result, closed with a rerun verdict
  naming its successor, shown closed
  with the successor recorded, the
  successor shown before it exists,
  opened as its own experiment, given a
  result, and closed with a keep
  verdict, the first experiment closed
  again and the successor opened again
  in refusal, a third experiment
  opened, given a result, and closed
  with a keep verdict naming a
  successor, and a final check reading
  the whole registry back. The amended
  run of record agrees on fourteen of
  fourteen cells, none failed, none
  blocked: the rerun close records the
  successor identically on both arms
  after the cut, the successor shown
  before it is opened refuses at exit 1
  on both arms, the successor opened as
  its own experiment stands in the
  registry identically after the cut,
  the repeat close and the repeat open
  refuse at exit 1 on both arms with
  stderr equal, and the keep verdict
  naming a successor refuses at exit 1
  on both arms with stderr equal. No
  conviction; the surface joins the
  standing regression's evidence. The
  registry after the successor's close
  holds exactly six lines on both arms,
  and after the final check the
  registry stands at exactly eight
  lines on both arms. The cut the close
  and post-close cells rely on is the
  VL-96 corpus discipline: the port's
  closes add a not_consulted judgment
  member the oracle predates,
  brace-matched, parsed, and verified
  to be exactly one of its two
  prescribed shapes before removal; a
  span of any other shape would have
  convicted. The driver's first run
  banked seven of fourteen with the
  seven post-close registry comparisons
  defined without the cut failing on
  exactly that span, a derivation miss
  in this round's cell definitions,
  which named the cut only for the
  close and read cells; the amendment
  extending the registry cut to every
  cell from the first close onward
  stands recorded in the lane's
  progress record before the re-run,
  and the amended run is the run of
  record.
- VL-115. Item three of the storefront
  standard stands re-verified through
  the API at this round's base: the
  description line stands
  byte-identical in the repository
  description, the Cargo.toml
  description field, and the README
  lede, all three read at this round's
  hand, and this round changes none of
  the three homes; the item stands
  passing, standing, not repaired.
- VL-116. Round 16 closes. Its items
  stand at VL-111, the read-back of
  Round 15's landing pull request, the
  shell in its true final form at the
  reading hand; VL-112, the standing
  regression re-run whole, the battery
  at twenty-five of twenty-five, the
  walk at ten of ten, the flighttape
  pin re-confirmed live with its suites
  green, and the oracle verified blob
  for blob; VL-113, the gate at the
  base tree and at the final tree;
  VL-114, the rerun successor at
  fourteen of fourteen cells agreeing,
  no conviction; and VL-115, the
  storefront's third item standing,
  re-verified, not repaired. The
  harness provenance stands here in its
  own text: the measurements ran on
  drivers kept outside the repo
  (battery16.py and fake_judge16.py,
  the battery driver renamed only in
  its round furniture from Round 15's
  and the fake judge copied byte for
  byte, walk16.py likewise renamed,
  successor16.py new this round, and
  verify_census16.py for the pin,
  oracle, and storefront reads). The
  measurement runs executed in the
  lane's staging against Round 13's
  chain binary (sha256
  4e5ee9dca8bcc591786db8576f11009e1b55a6a15015829f9a21cd1b4a018517,
  built from a checkout whose every
  source blob stands identical to this
  round's base) without a build-slot
  hold, because the drivers invoke no
  cargo work, on the VL-96 and VL-98
  precedent. The round's cargo work ran
  under the build slot: the base gate,
  the census, and the pin suite green
  in the chain's run under the slot:
  the chain's first launches waited out
  the box's closed write wave from
  02:05Z, the wave cleared in the box
  restart of about 04:02Z that killed
  the waiting launchers, and the chain
  relaunched into the open window
  acquired the slot at 04:26:40Z and
  ran every stage green; the final-tree
  gate at this lane's hand under the
  same slot discipline. The successor
  driver's first run banked seven of
  fourteen on a derivation miss in its
  own cell definitions, the amendment
  recorded in the progress record
  before the re-run, and the amended
  run is the run of record; the amended
  successor run's banking stands
  disclosed: its first attempts banked
  BLOCKED at setup in the closed wave,
  and one retry attempt's console
  showed fourteen of fourteen while its
  bank landed at zero bytes in the
  wave, which counts nowhere; the run
  of record landed at this lane's hand
  in the open window after the restart,
  its bank read back whole at 1,352
  bytes. Slice and closure staging
  stand in separate directories this
  round, as the standing hygiene
  charges, so both halves of the
  pushed-tree assertion stay
  re-performable from disk. The README
  records 16 of 100 in the closure
  commit written after this round's
  three reviews, fixes, and owner
  verification.
- VL-117. The read-back of Round 16's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 24 stands closed and
  merged: its body stands at 11,387
  characters, the lane's body with the
  owner completion record appended after
  it, its head
  6f22066c2b9710e9cec6a2d66f10e1769cbdf599
  over the base
  9a5af9eb6ba2a23b6efabc547eb507086d278a7c,
  and it landed through the merge queue
  as squash commit
  ecca672ea986e7f30475ef835a9714ff20ee2410,
  single parent
  9a5af9eb6ba2a23b6efabc547eb507086d278a7c,
  the landing tree
  09175438390b572053a778c4ba7ea2f1e9dbb977
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  ecca672e on a fresh read, its tree at
  09175438 over twenty-one blobs), and
  the README read on main at 16 of 100.
  The census at this read stands at four
  issue comments, no reviews, and no
  review comments: the trunk note
  7034971311 in its landed form (113
  characters, merged successfully), the
  CodeRabbit shell 7034976800 in its
  true final form at this read (11,264
  characters, created
  2026-11-20T08:56:34Z, last edited
  2026-11-20T09:00:18Z, Run ID
  077fbbbc-3a95-4beb-b5bf-cf8aafb5e081,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head 6f22066c,
  four pre-merge checks passed, no
  actionable comments generated), the
  merge comment 7035071853 (12
  characters), and the landing comment
  7037976204 (2,144 characters,
  2026-11-20T11:07:30Z) carrying this
  round's charges; nothing stands after
  the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with one Minor,
  standards APPROVE with the same Minor,
  red team APPROVE with no findings, the
  red team reproducing the battery, the
  amended successor surface, and the
  walk at its own hand with banks
  byte-identical to the lane's, its own
  build of the head tree never landing
  in the build slot, so its runs stand
  against the verified chain binary
  whose source is blob-identical to the
  head tree's code. The Minor rides here
  as the named erratum this item
  contracts: VL-113's sentences state
  the final tree's cargo test as
  forty-seven passed and none failed in
  the same split and the final-tree
  codepoint scan at exit 0, in the flat
  form of a measured result. The head
  gate did run at the final tree (cargo
  test, clippy, and fmt exits all 0,
  which under the gate's no-fail-fast
  form proves every suite passed), but
  its log pipes the test output through
  a tail window that shows only the
  registry suite's twenty, so no head
  record contains the split of three
  library, twenty-four judgment, and
  twenty registry; and the head gate's
  own scan stage crashed on a directory
  argument and recorded scan exit 1 in
  that log, the final tree's scan exit 0
  standing on the lane's separate
  per-file scan over the final tree's
  twenty-one files and confirmed at both
  review axes' own hands. The pull
  request body states the split's basis
  exactly (the split verified at the
  base gate on source identical to the
  final tree's, the head source
  blob-identical to the base), and no
  figure anywhere is false; the defect
  is the sentences' evidence basis,
  corrected here by name. This round's
  gate (VL-119) banks the full test
  result lines and a working per-file
  scan at both trees as the run of
  record, so no sentence of this round's
  rests on another tree's log or on a
  scan its own stage did not run.
- VL-118. The standing regression stands
  re-run whole at this round's hand. The
  fake-judge battery stands at
  twenty-five of twenty-five cells, the
  port against the oracle across the
  judged close: the unavailability cells
  at their contracted dispositions, C17
  naming an unpinned model on the choice
  call alone, C18 the choice answer
  malformed on the choice call alone,
  C19 the evidence answer malformed
  across choice and evidence, C20 the
  risk answer malformed across choice,
  evidence, and risk, C21 judged at the
  bounds with evidence ten and risk
  zero, and C22 the risk answer
  malformed across choice, evidence, and
  risk, exactly the dispositions VL-100
  contracted; the confidence boundary
  exact at both edges, the abstention
  cells refusing, the fixture replays
  byte-identical, the successor and
  margin cells at their standing
  outcomes. The registry-walk battery
  stands at ten of ten cells, five chain
  shapes across list and show on both
  arms: the 991 chain refused by name on
  both arms and both surfaces, the 992
  chain in the crash class on both, the
  port's streams empty, the oracle's
  recursion error named, the registry
  byte-unchanged after every refused or
  crashed view. The pin stands
  re-verified: the flighttape checkout
  at the pinned commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at fifty blobs, its suite re-run at
  241 tests across nineteen suites with
  none failed, and the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing in the source it was read
  from. The oracle stands re-verified:
  its tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  at twenty-four blobs, recomputed
  against the census saved at its
  extraction with none bad. The local
  gate at the base and head trees stands
  contracted at VL-119.
- VL-119. The local gate stands run at
  the base tree and at the head tree in
  the form this round contracts, each
  tree's figures read from that tree's
  own log. At the base tree (main at
  ecca672ea986e7f30475ef835a9714ff20ee2410,
  tree
  09175438390b572053a778c4ba7ea2f1e9dbb977,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines with
  no tail window anywhere in the chain,
  forty-seven passed and none failed,
  three library, twenty-four judgment,
  and twenty registry, read from the
  base gate's own log; cargo clippy with
  warnings denied exits 0 and cargo fmt
  --check exits 0; and the codepoint
  scan runs per file over the tree's
  twenty-one files with its planted
  control holding both dashes, the
  control reading one em dash and one en
  dash, every file at zero and zero, the
  totals at zero and zero, scan exit 0.
  At the head tree (the closure tree,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines the
  same way, forty-seven passed and none
  failed, three library, twenty-four
  judgment, and twenty registry, read
  from the head gate's own log; cargo
  clippy with warnings denied exits 0
  and cargo fmt --check exits 0; and the
  per-file codepoint scan reads its
  control at one and one, every file at
  zero and zero, the totals at zero and
  zero, scan exit 0. No figure in this
  item rests on another tree's log, and
  no scan figure rests on a stage that
  did not run: the form VL-113's
  sentences lacked, contracted by name
  at VL-117 and discharged here at both
  trees.
- VL-120. The no-result lifecycle stands
  measured fresh, a surface no earlier
  round measured: twin fresh directories
  on both arms, and fourteen cells in
  one fixed sequence, an experiment
  opened and closed with a keep verdict
  and no result recorded, shown, and
  offered a result after that close; a
  second experiment opened and closed
  with a rerun verdict naming a
  successor and no result recorded, the
  named successor then opened; a third
  experiment opened, given a first
  result and a second result, shown,
  closed with a keep verdict, and
  offered a result after its close; and
  a final check reading the whole
  registry back. The amended run of
  record agrees on fourteen of fourteen
  cells, none failed, none blocked: a
  close with no recorded result refuses
  at exit 1 on both arms, keep and rerun
  verdicts alike, the registry standing
  at one line after the keep attempt and
  at three after the rerun attempt; the
  result offered after the refused keep
  close lands at exit 0 on both arms,
  the experiment standing open, the
  registry at two lines; the successor
  named on the refused rerun close opens
  as its own experiment at exit 0 on
  both arms; the first and second
  results on the third experiment both
  land at exit 0 on both arms, the
  registry growing a line at each,
  results accumulating with the latest
  current as the README states; the
  close with results recorded lands at
  exit 0 on both arms; and the result
  offered after that landed close
  refuses at exit 1 on both arms with
  the registry byte-unchanged on each
  arm. After the final check the
  registry stands at exactly eight lines
  on both arms. No conviction; the
  surface joins the standing
  regression's evidence. The cut the
  close and post-close cells rely on is
  the VL-96 corpus discipline, with
  Round 16's amendment applied from the
  start: the registry comparison carries
  the cut on every cell from the first
  close onward, the port's closes adding
  a not_consulted judgment member the
  oracle predates, brace-matched,
  parsed, and verified to be exactly one
  of its two prescribed shapes before
  removal; a span of any other shape
  would have convicted. The driver's
  first run banked thirteen of fourteen,
  its one failure the unchanged flag on
  the result-after-close cell, a
  derivation miss in this round's cell
  definitions, which presumed the
  no-result close lands; the amendment
  removing that flag, the cell keeping
  its full cross-arm comparison, stands
  recorded in the lane's progress record
  before the re-run, and the amended run
  is the run of record.
- VL-121. Item three of the storefront
  standard stands re-verified through
  the API at this round's base: the
  description line stands byte-identical
  in the repository description, the
  Cargo.toml description field, and the
  README lede, all three read at this
  round's hand, and this round changes
  none of the three homes; the item
  stands passing, standing, not
  repaired.
- VL-122. This round's closure. The
  round's harness: the battery driver is
  Round 16's battery renamed only in its
  round furniture (its summary banner,
  its work directory, its bank's name),
  the fake judge is Round 16's fake
  judge byte for byte (md5
  a79ded49ae7285fd2d7b028c8b47dd7f), the
  walk driver is Round 16's walk renamed
  the same way, the lifecycle driver is
  new this round, and the census is this
  round's own; the instruments stand in
  the lane's scratch, not in the
  repository, and nothing in this round
  changes any code the repository ships.
  The measurement runs (the battery, the
  walk, the lifecycle surface) ran
  against Round 13's chain binary
  (sha256
  4e5ee9dca8bcc591786db8576f11009e1b55a6a15015829f9a21cd1b4a018517),
  whose source blobs stand identical to
  this round's base, the diff between
  Round 16's base tree and this round's
  base being exactly CONTRACT.md and
  README.md, without a build slot hold,
  on the precedent VL-96 and VL-98 set;
  every cargo run stood under the build
  slot. The round's weather stands
  disclosed: a write wave held content
  writes closed from about 11:07Z to
  about 11:24Z on 2026-11-20, and the
  base extraction and the instrument
  rename rode it out in retry loops
  inside the sitting, landing whole when
  the window opened; the base gate's
  chain took three holds inside the
  sitting, the first queueing two hours
  thirteen minutes and acquired, then
  killed by a box restart before its
  first stage landed, the second
  queueing one hour thirty-eight minutes
  and acquired, then dying inside its
  probe gate in a closed write wave at a
  second restart, the third queueing
  about seven minutes and running whole,
  and the head gate ran at the closure
  tree under the same slot, its figures
  at VL-119. The lifecycle surface's
  first run and its amendment stand at
  VL-120. The slice and the closure
  stand staged in separate directories,
  and the README's rounds line moves to
  17 of 100 in this closure commit only,
  after the reviews.
- VL-123. The read-back of Round 17's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 26 stands closed and
  merged: its body stands at 11,432
  characters, the lane's body with the
  owner completion record appended after
  it, its head
  68f531a420ca96cefcf7f9a10d95f852259d6f4e
  over the base
  ecca672ea986e7f30475ef835a9714ff20ee2410,
  and it landed through the merge queue
  as squash commit
  289037ddf73c99781f0754c875aa24998358dbe1,
  single parent
  ecca672ea986e7f30475ef835a9714ff20ee2410,
  the landing tree
  7af962e57334dddebfb64621fa0b3c36c3e042c0
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  289037dd on a fresh read, its tree at
  7af962e5 over twenty-one blobs), and
  the README read on main at 17 of 100.
  The census at this read stands at four
  issue comments, no reviews, and no
  review comments: the trunk note
  7046338618 in its landed form (113
  characters, merged successfully), the
  CodeRabbit shell 7046342369 in its
  true final form at this read (11,038
  characters, created
  2026-11-20T17:37:20Z, last edited
  2026-11-20T17:41:52Z, Run ID
  14dc2425-75d4-4f3a-8bf2-b2dee0f9a24c,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head 68f531a4,
  pre-merge checks passed, no actionable
  comments generated), the merge comment
  7046451171 (12 characters), and the
  landing comment 7046674663 (1,782
  characters, 2026-11-20T17:51:45Z)
  carrying this round's charges; nothing
  stands after the landing comment. The
  owner's completion record stands read
  with it: spec fidelity APPROVE,
  standards APPROVE, and red team
  APPROVE, no findings at any grade on
  any axis, the cleanest verdict set the
  bench has recorded. Its account stands
  contracted here: VL-119 discharged the
  gate form Round 16's Minor charged, in
  the substance at both trees, the full
  test result lines banked with no tail
  window anywhere in the chain and a
  working per-file codepoint scan as the
  run of record, the split of three
  library, twenty-four judgment, and
  twenty registry read from each tree's
  own log, verified from the actual logs
  at three hands; VL-117 contracted
  VL-113's flat final-tree sentences by
  name in the erratum shape, so the
  repair stands complete on both limbs,
  the record corrected and the practice
  changed; VL-120's amendment, its first
  run banked thirteen of fourteen on the
  lane's own derivation miss in the
  unchanged flag on the
  result-after-close cell, was judged
  strictly on the standing ordering
  signals and discharged, the first bank
  preserved, the amendment recorded in
  the lane's progress record before the
  re-run, and the disclosure standing in
  the item and the body; the red team
  reproduced the battery, the no-result
  lifecycle surface, and the walk whole
  at its own hand with banks
  byte-identical to the lane's on every
  line, re-ran the pinned crate's
  digest_cli suite clean under the build
  slot, and matched the closure tree
  twenty-one of twenty-one against a
  manifest it computed itself; and the
  standards axis's one ungraded
  observation, the shell's passed Title
  check remarking that the pull request
  title is long and dense, stands on the
  review record, the check passed and
  the remark non-actionable. This
  round's gate (VL-125) stands in the
  same form at both trees, the
  discharged VL-119 form carried forward
  as the standing form.
- VL-124. The standing regression stands
  re-run whole at this round's hand. The
  fake-judge battery stands at
  twenty-five of twenty-five cells, the
  port against the oracle across the
  judged close: the unavailability cells
  at their contracted dispositions, C17
  naming an unpinned model on the choice
  call alone, C18 the choice answer
  malformed on the choice call alone,
  C19 the evidence answer malformed
  across choice and evidence, C20 the
  risk answer malformed across choice,
  evidence, and risk, C21 judged at the
  bounds with evidence ten and risk
  zero, and C22 the risk answer
  malformed across choice, evidence, and
  risk, exactly the dispositions VL-100
  contracted; the confidence boundary
  exact at both edges, the abstention
  cells refusing, the fixture replays
  byte-identical, the successor and
  margin cells at their standing
  outcomes. The registry-walk battery
  stands at ten of ten cells, five chain
  shapes across list and show on both
  arms: the 991 chain refused by name on
  both arms and both surfaces, the 992
  chain in the crash class on both, the
  port's streams empty, the oracle's
  recursion error named, the registry
  byte-unchanged after every refused or
  crashed view. The pin stands
  re-verified: the flighttape checkout
  at the pinned commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at fifty blobs, its suite re-run at
  241 tests across nineteen suites with
  none failed, and the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing in the source it was read
  from. The oracle stands re-verified:
  its tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  at twenty-four blobs, recomputed
  against the census saved at its
  extraction with none bad. The local
  gate at the base and head trees stands
  contracted at VL-125.
- VL-125. The local gate stands run at
  the base tree and at the head tree in
  the form this round contracts, each
  tree's figures read from that tree's
  own log. At the base tree (main at
  289037ddf73c99781f0754c875aa24998358dbe1,
  tree
  7af962e57334dddebfb64621fa0b3c36c3e042c0,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines with
  no tail window anywhere in the chain,
  forty-seven passed and none failed,
  three library, twenty-four judgment,
  and twenty registry, read from the
  base gate's own log; cargo clippy with
  warnings denied exits 0 and cargo fmt
  --check exits 0; and the codepoint
  scan runs per file over the tree's
  twenty-one files with its planted
  control holding both dashes, the
  control reading one em dash and one en
  dash, every file at zero and zero, the
  totals at zero and zero, scan exit 0.
  At the head tree (the closure tree,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines the
  same way, forty-seven passed and none
  failed, three library, twenty-four
  judgment, and twenty registry, read
  from the head gate's own log; cargo
  clippy with warnings denied exits 0
  and cargo fmt --check exits 0; and the
  per-file codepoint scan reads its
  control at one and one, every file at
  zero and zero, the totals at zero and
  zero, scan exit 0. No figure in this
  item rests on another tree's log, and
  no scan figure rests on a stage that
  did not run: the VL-119 form,
  contracted forward at VL-123 and
  discharged here at both trees.
- VL-126. The interleaved registry
  stands measured fresh, a surface no
  earlier round measured: twin fresh
  directories on both arms, and fourteen
  cells in one fixed sequence, two
  experiments opened before either is
  given a result, their results
  interleaved, the first experiment
  shown open with its two results at
  positions three and five and then
  closed with a keep verdict, the second
  shown still open across that close and
  then closed with a kill verdict, a
  list over the mixed registry, a third
  experiment opened across the two
  closed experiments and given a result
  and shown, and a final check reading
  the whole registry back. The run of
  record agrees on fourteen of fourteen
  cells, none failed, none blocked, on
  its first run with no amendment: the
  first experiment's close rests on
  baseline position one and result
  position five on both arms, across the
  second experiment's open and result at
  positions two and four, with verdict
  keep and suggestion keep; the second
  experiment's close rests on baseline
  position two and result position four
  on both arms, both before the first
  experiment's close at position six,
  with verdict kill and suggestion kill;
  the third experiment stands opened at
  position eight with its result at
  position nine; and the registry stands
  at exactly nine lines on both arms
  after the final check. No conviction;
  the surface joins the standing
  regression's evidence. The cut the
  close and post-close cells rely on is
  the VL-96 corpus discipline, with
  Round 16's amendment applied from the
  start: the registry comparison carries
  the cut on every cell from the first
  close onward, the port's closes adding
  a not_consulted judgment member the
  oracle predates, brace-matched,
  parsed, and verified to be exactly one
  of its two prescribed shapes before
  removal; a span of any other shape
  would have convicted.
- VL-127. Item three of the storefront
  standard stands re-verified through
  the API at this round's base: the
  description line stands byte-identical
  in the repository description, the
  Cargo.toml description field, and the
  README lede, all three read at this
  round's hand, and this round changes
  none of the three homes; the item
  stands passing, standing, not
  repaired.
- VL-128. This round's closure. The
  round's harness: the battery driver is
  Round 17's battery renamed only in its
  round furniture (its summary banner,
  its work directory, its bank's name),
  the fake judge is Round 17's fake
  judge byte for byte (md5
  a79ded49ae7285fd2d7b028c8b47dd7f), the
  walk driver is Round 17's walk renamed
  the same way, the interleave driver is
  new this round, and the census is this
  round's own; the instruments stand in
  the lane's scratch, not in the
  repository, and nothing in this round
  changes any code the repository ships.
  The measurement runs (the battery, the
  walk, the interleaved registry) ran
  against Round 13's chain binary
  (sha256
  4e5ee9dca8bcc591786db8576f11009e1b55a6a15015829f9a21cd1b4a018517),
  whose source blobs stand identical to
  this round's base, the diff between
  Round 17's base tree and this round's
  base being exactly CONTRACT.md and
  README.md, without a build slot hold,
  on the precedent VL-96 and VL-98 set;
  every cargo run stood under the build
  slot. The round's weather stands
  disclosed: the base gate's chain took
  three holds inside the sitting, the
  first acquiring at 18:02:09Z on
  2026-11-20, verifying the base tree
  whole, and dying in its build stage
  when a write wave closed mid-build
  (rustc failing with an IO failure on
  its output stream, Operation not
  permitted, build exit 101; that hold's
  census and pin stages count nowhere),
  the second acquiring at 18:06:03Z and
  killed by a box restart inside its
  probe gate before any stage ran, the
  third acquiring at 18:15:58Z and
  running whole; the head gate's holds
  stand counted in the lane's progress
  record with its figures at VL-125. The
  interleaved registry's run of record
  stands at VL-126 on its first run, no
  amendment. The slice and the closure
  stand staged in separate directories,
  and the README's rounds line moves to
  18 of 100 in this closure commit only,
  after the reviews.
- VL-129. The read-back of Round 18's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 28 stands closed and
  merged: its body stands at 11,724
  characters, the lane's body with the
  owner completion record appended after
  it, its head
  e125871dc0d616790f07f0a890b964de721c5982
  over the base
  289037ddf73c99781f0754c875aa24998358dbe1,
  and it landed through the merge queue
  as squash commit
  a0cebc120d69e085f5f976960ebdd1b390be843e,
  single parent
  289037ddf73c99781f0754c875aa24998358dbe1,
  the landing tree
  d4db42c6ddb64ee06b4330b46619d01a8abfe430
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  a0cebc12 on a fresh read, its tree at
  d4db42c6 over twenty-one blobs), and
  the README read on main at 18 of 100.
  The census at this read stands at four
  issue comments, no reviews, and no
  review comments: the trunk note
  7048752574 in its landed form (113
  characters, merged successfully), the
  CodeRabbit shell 7048756858 in its
  true final form at this read (11,048
  characters at this hand, the lane
  having read 11,046 in its finished
  form, the shell's two-character
  re-render drift standing between the
  reads; created 2026-11-20T19:32:16Z,
  last edited 2026-11-20T19:36:02Z, Run
  ID
  b42f1902-ff57-4949-968e-67162ada5bbf,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head e125871d,
  pre-merge checks passed, no actionable
  comments generated), the merge comment
  7048839392 (12 characters), and the
  landing comment 7049234042 (1,875
  characters, 2026-11-20T19:56:05Z)
  carrying this round's charges; nothing
  stands after the landing comment. The
  owner's completion record stands read
  with it: spec fidelity APPROVE,
  standards APPROVE, and red team
  APPROVE, no findings at any grade on
  any axis, the second consecutive round
  on this repo with a clean verdict set.
  Its account stands contracted here:
  VL-126's interleaved registry ran
  clean on its first run, fourteen of
  fourteen with no amendment, and the
  first-run claim was sustained against
  the disk itself at two review hands,
  the spec axis and the red team each
  finding exactly one driver and one
  bank in the lane record with no
  earlier partial anywhere; VL-119's
  gate form discharged a second round
  running, in the substance at both
  trees, the full test result lines
  banked with no tail window anywhere in
  the chain and a working per-file
  codepoint scan as the run of record,
  the split of three library,
  twenty-four judgment, and twenty
  registry read from each tree's own
  log, verified from the actual logs at
  three hands; the red team's one
  carried limitation stands stated as it
  stands, its own digest_cli re-run
  never landing, a forty-try retry hold
  on the build slot never acquiring, so
  that leg stands unrun at its hand and
  the adopted figure rests on the lane's
  pin log re-summed at its hand, with
  the golden constant standing in the
  pinned checkout's digest_cli.rs,
  identity.rs, and convert.rs, read in
  its own clean copy of the checkout;
  and the standards axis's one ungraded
  observation, the shell's passed Title
  check remarking that the title is long
  and lists identifiers, only partially
  related and overly broad, while
  passing, stands on the review record,
  the check passed and the remark
  non-actionable. This round's gate
  (VL-131) stands in the same form at
  both trees, the discharged VL-119 form
  carried forward as the standing form.
- VL-130. The standing regression stands
  re-run whole at this round's hand. The
  fake-judge battery stands at
  twenty-five of twenty-five cells, the
  port against the oracle across the
  judged close: the unavailability cells
  at their contracted dispositions, C17
  naming an unpinned model on the choice
  call alone, C18 the choice answer
  malformed on the choice call alone,
  C19 the evidence answer malformed
  across choice and evidence, C20 the
  risk answer malformed across choice,
  evidence, and risk, C21 judged at the
  bounds with evidence ten and risk
  zero, and C22 the risk answer
  malformed across choice, evidence, and
  risk, exactly the dispositions VL-100
  contracted; the confidence boundary
  exact at both edges, the abstention
  cells refusing, the fixture replays
  byte-identical, the successor and
  margin cells at their standing
  outcomes. The registry-walk battery
  stands at ten of ten cells, five chain
  shapes across list and show on both
  arms: the 991 chain refused by name on
  both arms and both surfaces, the 992
  chain in the crash class on both, the
  port's streams empty, the oracle's
  recursion error named, the registry
  byte-unchanged after every refused or
  crashed view. The pin stands
  re-verified: the flighttape checkout
  at the pinned commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at fifty blobs, its suite re-run at
  241 tests across nineteen suites with
  none failed, and the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing in the source it was read
  from. The oracle stands re-verified:
  its tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  at twenty-four blobs, recomputed
  against the census saved at its
  extraction with none bad. The local
  gate at the base and head trees stands
  contracted at VL-131.
- VL-131. The local gate stands run at
  the base tree and at the head tree in
  the form this round contracts, each
  tree's figures read from that tree's
  own log. At the base tree (main at
  a0cebc120d69e085f5f976960ebdd1b390be843e,
  tree
  d4db42c6ddb64ee06b4330b46619d01a8abfe430,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines with
  no tail window anywhere in the chain,
  forty-seven passed and none failed,
  three library, twenty-four judgment,
  and twenty registry, read from the
  base gate's own log; cargo clippy with
  warnings denied exits 0 and cargo fmt
  --check exits 0; and the codepoint
  scan runs per file over the tree's
  twenty-one files with its planted
  control holding both dashes, the
  control reading one em dash and one en
  dash, every file at zero and zero, the
  totals at zero and zero, scan exit 0.
  At the head tree (the closure tree,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines the
  same way, forty-seven passed and none
  failed, three library, twenty-four
  judgment, and twenty registry, read
  from the head gate's own log; cargo
  clippy with warnings denied exits 0
  and cargo fmt --check exits 0; and the
  per-file codepoint scan reads its
  control at one and one, every file at
  zero and zero, the totals at zero and
  zero, scan exit 0. No figure in this
  item rests on another tree's log, and
  no scan figure rests on a stage that
  did not run: the VL-119 form,
  contracted forward at VL-129 and
  discharged here at both trees.
- VL-132. The partial and mixed measures
  surface stands measured fresh, a
  surface no earlier round measured:
  twin fresh directories on both arms,
  and fourteen cells in one fixed
  sequence, four experiments judged on
  cost lower and rescue higher against
  the same baseline, closed over four
  result shapes, a result omitting the
  rescue measure, a result improving
  cost and regressing rescue, a result
  carrying an extra measure no judged
  entry names beside a full improvement,
  and a result repeating the baseline
  exactly. The run of record agrees on
  fourteen of fourteen cells, none
  failed, none blocked, on its first run
  with no amendment: the first
  experiment's close rests on baseline
  position one and result position two
  on both arms, with regressed empty and
  unmeasured naming rescue, the
  suggestion keep; the second
  experiment's close rests on baseline
  position four and result position five
  on both arms, with regressed naming
  rescue and unmeasured empty, the
  suggestion rerun against the kill
  verdict, the close landing with no
  judge named, and its rests_on and
  regressed read back on the show
  surface; the third experiment's close
  rests on baseline position seven and
  result position eight on both arms,
  with regressed and unmeasured both
  empty, the extra measure riding in the
  record; the fourth experiment's close
  rests on baseline position ten and
  result position eleven on both arms,
  with regressed and unmeasured both
  empty over the zero-movement result;
  and the registry stands at exactly
  twelve lines on both arms after the
  final check. No conviction; the
  surface joins the standing
  regression's evidence. The cut the
  close and post-close cells rely on is
  the VL-96 corpus discipline, with
  Round 16's amendment applied from the
  start: the registry comparison carries
  the cut on every cell from the first
  close onward, the port's closes adding
  a not_consulted judgment member the
  oracle predates, brace-matched,
  parsed, and verified to be exactly one
  of its two prescribed shapes before
  removal; a span of any other shape
  would have convicted.
- VL-133. Item three of the storefront
  standard stands re-verified through
  the API at this round's base: the
  description line stands byte-identical
  in the repository description, the
  Cargo.toml description field, and the
  README lede, all three read at this
  round's hand, and this round changes
  none of the three homes; the item
  stands passing, standing, not
  repaired.
- VL-134. This round's closure. The
  round's harness: the battery driver is
  Round 18's battery renamed only in its
  round furniture (its summary banner,
  its work directory, its bank's name),
  the fake judge is Round 18's fake
  judge byte for byte (md5
  a79ded49ae7285fd2d7b028c8b47dd7f), the
  walk driver is Round 18's walk renamed
  the same way, the partial driver is
  new this round, and the census is this
  round's own; the instruments stand in
  the lane's scratch, not in the
  repository, and nothing in this round
  changes any code the repository ships.
  The measurement runs (the battery, the
  walk, the partial and mixed measures)
  ran against Round 13's chain binary
  (sha256
  4e5ee9dca8bcc591786db8576f11009e1b55a6a15015829f9a21cd1b4a018517),
  whose source blobs stand identical to
  this round's base, the diff between
  Round 18's base tree and this round's
  base being exactly CONTRACT.md and
  README.md, without a build slot hold,
  on the precedent VL-96 and VL-98 set;
  every cargo run stood under the build
  slot. The round's weather stands
  disclosed: a write wave stood closed
  at the lane's seating, exec writes
  refused with Operation not permitted
  from about 19:57Z on 2026-11-20, the
  read-back first taken by a stdout-only
  pass and its banking landed when the
  wave opened after a box restart at
  20:18Z; the base gate's chain took two
  holds inside the sitting, the first
  queueing on the build slot from
  20:22:32Z for about an hour without
  acquiring and killed by SIGTERM in a
  box restart with no stage run and no
  log written, the second queueing from
  21:24:08Z, acquiring at 21:27:34Z, and
  running whole; the head gate's holds
  stand counted in the lane's progress
  record with its figures at VL-131. The
  partial and mixed measures run of
  record stands at VL-132 on its first
  run, no amendment. The slice and the
  closure stand staged in separate
  directories, and the README's rounds
  line moves to 19 of 100 in this
  closure commit only, after the
  reviews.
- VL-135. The read-back of Round 19's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 30 stands closed and
  merged: its body stands at 15,036
  characters, the lane's body with its
  step 0 section and the CodeRabbit
  release-notes appendix and the owner
  completion record appended after it,
  its head
  971db36c7753390345b5a89ddb9c0e737587c478
  over the base
  a0cebc120d69e085f5f976960ebdd1b390be843e,
  and it landed through the merge queue
  as squash commit
  bcf9e9ad4ba668c1627606b71fa4eb7e6b097766,
  single parent
  a0cebc120d69e085f5f976960ebdd1b390be843e,
  the landing tree
  ba970c619306b5de10edf60b0548c8dc2c903a7d
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  bcf9e9ad on a fresh read, its tree at
  ba970c61 over twenty-one blobs), and
  the README read on main at 19 of 100.
  The census at this read stands at four
  issue comments, no reviews, and no
  review comments: the trunk note
  7056633277 in its landed form (113
  characters, merged successfully), the
  CodeRabbit shell 7056635647 in its
  true final form at this read (10,945
  characters at this hand, the lane
  having read 10,943 in its finished
  form, the shell's two-character
  re-render drift standing between the
  reads; created 2026-11-21T03:29:02Z,
  last edited 2026-11-21T03:33:35Z, Run
  ID
  8e411ad7-73e9-403a-83b1-55e9767fba54,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head 971db36c,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline
  comments), the merge comment
  7056687267 (12 characters), and the
  landing comment 7057608843 (2,201
  characters, 2026-11-21T04:59:30Z)
  carrying this round's charges; nothing
  stands after the landing comment. The
  owner's completion record stands read
  with it: spec fidelity APPROVE with no
  findings at any grade, standards
  APPROVE with no findings at any grade,
  and red team APPROVE with one Minor, a
  carried environment limitation, not a
  defect. Its account stands contracted
  here: the round ran under two lanes in
  succession, the first banking the
  read-back, the whole regression, the
  fresh surface, the base gate, and the
  staging before erroring on a transport
  failure at 2026-11-20T23:26Z, the
  continuation lane adopting the banked
  record, re-verifying the staging at
  its own hand, running the head gate
  through four holds, and pushing, the
  first lane having pushed nothing;
  VL-132's partial and mixed measures
  ran clean on its first run, fourteen
  of fourteen with no amendment, the
  second consecutive first-run-clean
  surface on this repo, and the
  first-run claim was sustained against
  the disk itself at two review hands,
  the spec axis and the red team each
  finding exactly one driver and one
  bank in the lane record with no
  earlier partial anywhere, timestamps
  in single-clean-run order; the VL-119
  gate form discharged a third round
  running, in the substance at both
  trees, the full test result lines
  banked with no tail window anywhere in
  the chain and a working per-file
  codepoint scan as the run of record,
  the split of three library,
  twenty-four judgment, and twenty
  registry read from each tree's own
  log, verified from the actual logs at
  three hands, the head gate's three
  dead holds counted in the body with no
  figure resting on them; the scan
  instrument's home directory was
  removed by parties unknown after the
  base gate ran, the continuation lane
  reconstructed the instrument from its
  banked behavior in the base gate's
  log, and the red team proved the
  reconstruction at its own hand, its
  output byte-identical against the base
  gate's banked scan section at 828
  bytes, the control at one and one, the
  totals at zero and zero, exit 0; and
  the red team's one Minor stands stated
  as it stands, the box-wide write fault
  shut for its entire review window,
  fsync refused at every probe from
  03:40Z to 04:53Z, so its run-side
  reproductions of the battery, the
  fresh surface, and the walk, and its
  own digest_cli re-run, stand unrun at
  its hand, the same posture as Round
  18's carried limitation and wider in
  scope, those legs resting on the
  lane's banks, which the red team read
  whole and matched cell for cell
  against the contracted dispositions,
  while what it verified firsthand
  stands at full strength, both trees
  matched twenty-one of twenty-one
  against its own manifest, the scanner
  proof byte-identical, both gate logs
  re-derived from their own lines, the
  pin log re-summed at 241 across 19
  suites with the golden constant
  standing in source, and the hold
  accounting verified line for line; and
  the standards axis's one ungraded
  observation, the shell's passed Title
  check remarking that the title is long
  and uses internal identifiers while
  passing, stands on the review record,
  the check passed and the remark
  non-actionable. This round's gate
  (VL-137) stands in the same form at
  both trees, the discharged VL-119 form
  carried forward as the standing form.
- VL-136. The standing regression stands
  re-run whole at this round's hand. The
  fake-judge battery stands at
  twenty-five of twenty-five cells, the
  port against the oracle across the
  judged close: the unavailability cells
  at their contracted dispositions, C17
  naming an unpinned model on the choice
  call alone, C18 the choice answer
  malformed on the choice call alone,
  C19 the evidence answer malformed
  across choice and evidence, C20 the
  risk answer malformed across choice,
  evidence, and risk, C21 judged at the
  bounds with evidence ten and risk
  zero, and C22 the risk answer
  malformed across choice, evidence, and
  risk, exactly the dispositions VL-100
  contracted; the confidence boundary
  exact at both edges, the abstention
  cells refusing, the fixture replays
  byte-identical, the successor and
  margin cells at their standing
  outcomes. The registry-walk battery
  stands at ten of ten cells, five chain
  shapes across list and show on both
  arms: the 991 chain refused by name on
  both arms and both surfaces, the 992
  chain in the crash class on both, the
  port's streams empty, the oracle's
  recursion error named, the registry
  byte-unchanged after every refused or
  crashed view. The pin stands
  re-verified: the flighttape checkout
  at the pinned commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at fifty blobs, its suite re-run at
  241 tests across nineteen suites with
  none failed, and the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing in the source it was read
  from. The oracle stands re-verified:
  its tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  at twenty-four blobs, recomputed
  against the census saved at this
  round's own extraction with none bad,
  this round extracting its own copy
  because the Round 13 extraction's home
  was removed in Round 19. The local
  gate at the base and head trees stands
  contracted at VL-137.
- VL-137. The local gate stands run at
  the base tree and at the head tree in
  the form this round contracts, each
  tree's figures read from that tree's
  own log. At the base tree (main at
  bcf9e9ad4ba668c1627606b71fa4eb7e6b097766,
  tree
  ba970c619306b5de10edf60b0548c8dc2c903a7d,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines with
  no tail window anywhere in the chain,
  forty-seven passed and none failed,
  three library, twenty-four judgment,
  and twenty registry, read from the
  base gate's own log; cargo clippy with
  warnings denied exits 0 and cargo fmt
  --check exits 0; and the codepoint
  scan runs per file over the tree's
  twenty-one files with its planted
  control holding both dashes, the
  control reading one em dash and one en
  dash, every file at zero and zero, the
  totals at zero and zero, scan exit 0.
  At the head tree (the closure tree,
  its twenty-one blobs verified in the
  checkout before the gate ran): cargo
  test banks its full result lines the
  same way, forty-seven passed and none
  failed, three library, twenty-four
  judgment, and twenty registry, read
  from the head gate's own log; cargo
  clippy with warnings denied exits 0
  and cargo fmt --check exits 0; and the
  per-file codepoint scan reads its
  control at one and one, every file at
  zero and zero, the totals at zero and
  zero, scan exit 0. No figure in this
  item rests on another tree's log, and
  no scan figure rests on a stage that
  did not run: the VL-119 form,
  contracted forward at VL-135 and
  discharged here at both trees. The
  base chain's holds stand counted at
  VL-140; the head gate's holds stand
  counted in the lane's progress record.
- VL-138. The resting result surface
  stands measured fresh, a surface no
  earlier round measured: twin fresh
  directories on both arms, and nineteen
  cells in one fixed sequence, four
  experiments judged on cost lower and
  rescue higher against the same
  baseline, each holding two or more
  landed results at its close, so the
  close's resting place among several
  results is the measured shape. The run
  of record agrees on nineteen of
  nineteen cells, none failed, none
  blocked, on its first run with no
  amendment: the first experiment's
  close rests on baseline position one
  and result position three on both
  arms, with regressed and unmeasured
  both empty and the suggestion keep,
  its first result's rescue regression
  leaving no trace in the close; the
  second experiment's close rests on
  baseline position five and result
  position seven on both arms, with
  regressed naming rescue and unmeasured
  empty, the suggestion rerun against
  the kill verdict, the close landing
  with no judge named, and its rests_on
  and regressed read back on the show
  surface; the third experiment's close
  rests on baseline position nine and
  result position twelve on both arms,
  with regressed and unmeasured both
  empty and the suggestion keep, its
  middle result's double regression
  leaving no trace; the fourth
  experiment's close rests on baseline
  position fourteen and result position
  sixteen on both arms, with regressed
  and unmeasured both empty, its first
  result's omitted rescue measure not
  carrying into the close; and the
  registry stands at exactly seventeen
  lines on both arms after the final
  check. No conviction; the surface
  joins the standing regression's
  evidence. The cut the close and
  post-close cells rely on is the VL-96
  corpus discipline, with Round 16's
  amendment applied from the start: the
  registry comparison carries the cut on
  every cell from the first close
  onward, the port's closes adding a
  not_consulted judgment member the
  oracle predates, brace-matched,
  parsed, and verified to be exactly one
  of its two prescribed shapes before
  removal; a span of any other shape
  would have convicted.
- VL-139. Item three of the storefront
  standard stands re-verified through
  the API at this round's base: the
  description line stands byte-identical
  in the repository description, the
  Cargo.toml description field, and the
  README lede, all three read at this
  round's hand, and this round changes
  none of the three homes; the item
  stands passing, standing, not
  repaired.
- VL-140. This round's closure. The
  round's harness: the battery driver is
  Round 19's battery renamed only in its
  round furniture (its summary banner,
  its work directory, its bank's name,
  its item number), the fake judge is
  Round 19's fake judge byte for byte
  (md5
  a79ded49ae7285fd2d7b028c8b47dd7f), the
  walk driver is Round 19's walk renamed
  the same way, the rests driver is new
  this round, and the census is this
  round's own; the instruments stand in
  the lane's scratch, not in the
  repository, and nothing in this round
  changes any code the repository ships.
  The measurement runs (the battery, the
  walk, the resting result) ran against
  the chain binary built fresh from the
  blob-verified base checkout in this
  lane's own chain (sha256
  b66790055a8875ccd9e59ad529202f63d9243239754945dc7383bd35b2a8c055),
  the Round 13 binary's home directory
  having been removed in Round 19; every
  cargo run stood under the build slot.
  The round's weather stands disclosed:
  a write wave stood shut at the lane's
  seating, exec writes refused with
  Operation not permitted from about
  05:01Z on 2026-11-21, the read-back
  first taken by a stdout-only pass, its
  banking and the base and oracle
  extractions landed in a short open
  window after a box restart at 05:29Z,
  and the wave shut again until a second
  restart opened it at 10:51Z; the base
  chain took three holds inside the
  sitting, the first queueing from
  05:32:26Z, acquiring at 08:31:43Z
  after one hundred seventy-nine minutes
  queued, and dying at its probe gate
  with two hundred forty tries exhausted
  and no stage run, the second launched
  at 09:32:07Z and killed queued by the
  second box restart at about 10:50Z
  with no stage run, the third queueing
  from 10:51:38Z, acquiring at
  10:54:42Z, and running whole; the head
  gate's holds stand counted in the
  lane's progress record with its
  figures at VL-137. The resting result
  run of record stands at VL-138 on its
  first run, no amendment. The slice and
  the closure stand staged in separate
  directories, and the README's rounds
  line moves to 20 of 100 in this
  closure commit only, after the
  reviews.
- VL-141. The read-back of Round 20's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 32 stands closed and
  merged: its body stands at 15,322
  characters, the lane's body with its
  hosted line, its step 0 section, and
  the CodeRabbit release-notes appendix
  and the owner completion record
  appended after it, its head
  f06498d55a3a024b8fdeb65dddb634c35e4bf778
  over the base
  bcf9e9ad4ba668c1627606b71fa4eb7e6b097766,
  and it landed through the merge queue
  as squash commit
  286c0ef56590372fd7310222025026451bc2b733,
  single parent
  bcf9e9ad4ba668c1627606b71fa4eb7e6b097766,
  the landing tree
  74168f98c6bbcff50cb83417d05469a582e449f3
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  286c0ef5 on a fresh read, its tree at
  74168f98 over twenty-one blobs, not
  truncated), and the README read on
  main at 20 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7065857549 in its landed
  form (113 characters, merged
  successfully), the CodeRabbit shell
  7065860164 cited in drift-aware form
  (11,017 characters at this hand, edit
  stamp 2026-11-21T16:48:03Z; the Round
  20 lane read 11,015 characters in its
  finished form, the edit stamp
  2026-11-21T16:45:11Z standing in its
  step 0 section, the shell's
  two-character re-render drift standing
  between the reads and the re-render
  carrying the later stamp; created
  2026-11-21T16:43:13Z, Run ID
  70fa28d3-0a83-474b-86e1-a7420692fe1d,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head f06498d5,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline comments;
  the shell read whole at this round's
  hand), the merge comment 7065934564
  (12 characters), and the landing
  comment 7066906676 (2,295 characters,
  2026-11-21T17:58:31Z) carrying this
  round's charges; nothing stands after
  the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with one Minor,
  standards APPROVE with one Minor, and
  red team APPROVE with one Minor, no
  Blocker, no Major, and no defect in
  the round at any grade on any axis.
  Its account stands contracted here:
  the standards Minor is a numeral
  correction the owner made in the
  record itself, the body's Disclosures
  stating that no figure rests on the
  five dead holds where the round's own
  accounting, corroborated line for line
  by both chain status files at two
  review hands, gives six holds in all,
  the base chain three and the head gate
  three, two of which ran whole, so the
  dead holds are four, not five; the
  substantive claim stands true, every
  figure re-deriving from the two whole
  holds' logs, re-derived independently
  by the standards axis and the red
  team, and the CONTRACT does not repeat
  the error; four is the count that sums
  from the stated parts, and this
  round's read-back contracts the
  corrected figure. The spec Minor and
  the standards axis's bounded reading
  are the same fact, the shell's
  two-character drift between the lane's
  read and the owner's, the same class
  VL-135 contracts for PR 30's shell,
  and this round cites shells in
  drift-aware form, the reading hand and
  the edit stamp named, as the census
  above does. VL-138's fresh surface,
  the resting result, ran clean on its
  first run, nineteen of nineteen with
  no amendment, the third consecutive
  first-run-clean surface on this repo,
  and the first-run claim was attacked
  against the disk at two review hands
  and held both times, each finding
  exactly one driver composed before the
  run, one bank, and one work directory,
  timestamps in single-clean-run order;
  the VL-119 gate form discharged a
  fourth round running, in the substance
  at both trees, the full test result
  lines banked with no tail window
  anywhere in the chain and a working
  per-file codepoint scan as the run of
  record, the split of three library,
  twenty-four judgment, and twenty
  registry read from each tree's own
  log. And the red team's one Minor
  stands stated as it stands, a
  limitation, not a defect: the write
  fault stood shut for its entire review
  window, fsync refused at its probes at
  16:50Z, 16:58Z, and 17:02Z on
  2026-11-21, so its run-side
  reproductions stand unrun at its hand,
  the rests re-run blocked at setup
  twice on a verified driver copy, the
  walk re-run's seeds refused on
  md5-verified copies, its digest_cli
  re-run under the acquired slot failing
  on EPERM fingerprint writes, and the
  battery re-run not separately
  attempted through the same
  demonstrated-refused mechanism; those
  legs rest on the lane's banks, which
  the red team read whole and matched
  cell for cell against the contracted
  dispositions, and on the surface
  registries read whole on both arms,
  while what it verified firsthand
  stands at full strength, both trees
  matched twenty-one of twenty-one
  against its own manifest, the chain
  binary hashed at its hand to the
  claimed sha256, both gate logs
  re-derived from their own lines, the
  pin log re-summed at 241 across 19
  suites with the golden constant
  standing in source, and the hold
  accounting verified line for line; and
  the standards axis's one ungraded
  observation, the shell's passed Title
  check remarking that the title is
  partially related, dense with
  identifiers, and overly broad for a
  scanning reader while passing, stands
  on the review record, the check passed
  and the remark non-actionable. This
  round's gate (VL-143) stands in the
  same form at both trees, the
  discharged VL-119 form carried forward
  as the standing form.
- VL-142. The standing regression stands
  re-run whole at this round's hand
  against the chain binary built fresh
  from the blob-verified base checkout
  under the slot (sha256
  50f0ddd1a8627a6b00b04b9c108ea40711ae0b17e9564e2bd554c642e239aa48).
  The judgment battery stands at 25 of
  25 cells passing, the fake judge
  copied byte for byte from Round 20's
  instrument (md5
  a79ded49ae7285fd2d7b028c8b47dd7f), and
  cells C17 to C22 stand at VL-100's
  dispositions: C17 closes with the
  judge unavailable, its reason naming
  an unpinned model; C18 with the choice
  answer malformed; C19 with the
  evidence answer malformed; C20 with
  the risk answer malformed; C21's
  bounds hold at scores of 10 and 0; and
  C22's risk answer at 105 is malformed,
  each cell's calls and reasons matching
  the contracted text. The walk stands
  at 10 of 10 cells passing against the
  oracle. The pin stands re-verified:
  flighttape at commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  tree
  afd4d7d13520411e003762548fc79082a1b9cf6e,
  its 50 blobs recomputed at this
  round's census, and its suite re-run
  at 241 tests across 19 suites, none
  failed, the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing in source; the pin's run of
  record is this lane's own hand run
  under the slot at exit 0, its two
  earlier attempts struck by the
  weather, the chain's attempt refused
  by rustc's writes when the wave
  returned (exit 101) and the recovery's
  attempt failing on a registry read in
  the second it acquired (exit 101),
  both logs banked and neither resting
  under any figure. The oracle stands
  re-verified: commit
  d62e5dc9547295f9ea7980f8298d905767b22f14,
  tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21,
  its 24 blobs recomputed at the census,
  and both twin arms of every
  measurement below ran against this
  extraction.
- VL-143. The local gate stands run at
  both trees in the VL-119 form, the
  full cargo test lines banked at each
  tree with no tail window. At the base
  tree (74168f98), under the chain's
  hold: cargo test 47 passed as 3
  library, 24 judgment, and 20 registry,
  read from that tree's own log, test
  exit 0; clippy over all targets with
  warnings denied, exit 0; fmt check,
  exit 0; and the per-file codepoint
  scan as the run of record, its planted
  control holding both dashes reading 1
  em dash and 1 en dash, the tree's
  totals 0 and 0, scan exit 0. At the
  head tree, the closure tree, under its
  own hold in the same form: cargo test
  47 passed as 3 library, 24 judgment,
  and 20 registry, read from that tree's
  own log, test exit 0; clippy exit 0;
  fmt exit 0; the scan's control reading
  1 and 1, totals 0 and 0, scan exit 0.
  The gate's holds this round: the base
  chain's first hold queued at 18:31:03Z
  on 2026-11-21 and died in the runtime
  restart at about 20:24Z without
  acquiring, one dead hold that ran no
  work; its second hold acquired at
  20:37:22Z and ran the base gate and
  the census whole; the head gate's
  first launch queued at 22:41Z and was
  killed by the lane in the queue when
  the wave's return broke its checkout
  copy, a dead hold that ran no work;
  its first relaunch queued at 00:28Z on
  2026-11-22 on the rebuilt tree and
  died in the runtime restart at about
  02:26Z without acquiring, a further
  dead hold; its second relaunch
  acquired at 04:24Z on 2026-11-22 and
  died in the runtime restart moments
  later, before any gate output banked,
  a further dead hold; its third
  relaunch acquired at 05:53Z on
  2026-11-22 and died the same way, a
  further dead hold; and its fourth
  relaunch acquired and ran whole. No
  figure in this item rests on another
  tree's log or on a dead hold.
- VL-144. The fresh surface is the named
  margins, proposed in the lane record
  before it ran: whether a margin named
  at three levels, on the open, on a
  measure's baseline entry, and on a
  measure's result entry, governs the
  regression call the same way in the
  port and in the oracle, under the rule
  that a measure's own margin stands
  first, then the open's margin, then
  ten percent, that a measure regresses
  when its adverse relative move in
  percent is strictly greater than the
  margin in force, and that a zero
  baseline leaves the margin
  unconsulted. The surface stands at 26
  of 26 cells agreeing on its first run,
  no amendment, the fourth consecutive
  first-run-clean surface on this repo.
  Eight experiments close on both arms
  at the contracted positions, (1, 2),
  (4, 5), (7, 8), (10, 11), (13, 14),
  (16, 17), (19, 20), and (22, 23):
  exp-a keeps with no regression, its 15
  percent move inside the named 20 and
  past the default 10, so the open's
  margin governs; exp-b names cost
  regressed and suggests rerun, its 30
  percent move past the named 20, the
  judge's verdict keep standing
  contested against the suggestion;
  exp-c keeps with no regression, the
  cost entry's margin of 30 governing
  over the open's 5 on a 20 percent
  move, so the per-measure margin stands
  first; exp-d names cost regressed and
  suggests rerun, the cost entry's
  margin of 5 governing over the open's
  50 on a 10 percent move, the verdict
  rerun agreeing; exp-e keeps, its move
  of exactly 25 percent at a margin of
  25, at is not past; exp-f names cost
  regressed and suggests rerun, its 26
  percent move one past the margin of
  25, the judge's verdict kill standing
  contested; exp-g keeps, the rescue
  measure's own float margin of 12.5 met
  exactly by a 12.5 percent move in the
  higher direction, the per-measure
  margin standing alone with no open
  margin named; and exp-h names cost
  regressed and suggests rerun on a zero
  cost baseline, the named margin of 90
  unconsulted and the rested value
  positive, the judge's verdict keep
  standing contested. Every cell exits 0
  on both arms, the final registry
  stands at 24 lines on both arms, and
  the registries agree line for line
  after the standing cut from the first
  close onward.
- VL-145. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-146. This round closes here. Round
  22 is charged with the read-back of
  this round's pull request on its
  closed record, the standing regression
  and the gate in their standing forms,
  a fresh twin surface Rounds 14 through
  21 did not measure, and storefront
  item 3 verified again. The round's
  whole hold census: nine holds in all,
  four acquired and ran, five dead, and
  the count sums from the stated parts.
  Hold 1, the base chain's first, queued
  at 18:31:03Z on 2026-11-21 and died in
  the runtime restart at about 20:24Z
  without acquiring; hold 2 acquired at
  20:37:22Z and ran the base verify, the
  base build, the base gate, and the
  census whole, its pin step struck by
  the wave's return at 20:39Z; hold 3,
  the recovery's, acquired at 22:26:35Z,
  its pin attempt failing on a registry
  read while its measurement legs, the
  battery, the walk, and the named
  margins, ran whole against the chain
  binary; hold 4, the pin's run of
  record at this lane's hand, acquired
  and ran whole at exit 0; hold 5, the
  head gate's first launch, queued at
  22:41Z and was killed by the lane in
  the queue when the wave's return broke
  its checkout copy, the round's second
  dead hold; hold 6, the head gate's
  first relaunch on the rebuilt tree,
  queued at 00:28Z on 2026-11-22 and
  died in the runtime restart at about
  02:26Z without acquiring, the round's
  third dead hold; hold 7, the head
  gate's second relaunch, acquired at
  04:24Z on 2026-11-22 and died in the
  runtime restart moments later, before
  any gate output banked, the round's
  fourth dead hold; hold 8, the head
  gate's third relaunch, acquired at
  05:53Z on 2026-11-22 and died in the
  runtime restart moments later, before
  any gate output banked, the round's
  fifth dead hold; and hold 9, the head
  gate's fourth relaunch, acquired and
  ran whole. The recovery's first
  attempt died probing in the same
  restart that killed hold 1, before it
  ever reached the slot, and counts as a
  dead recovery attempt, not a hold. The
  weather stands disclosed: the write
  fault stood shut from the lane's
  seating until about 18:26Z, opened for
  the banking and the extractions,
  returned at the base chain's pin step,
  and stood shut again until the restart
  at about 22:24Z, after which it stood
  open for the recovery and the pin's
  run of record, flickered shut once
  more at 22:41Z long enough to break
  the head gate's first checkout copy,
  and stood open for the head gate's
  fourth relaunch on the rebuilt tree.
  Two instrument disclosures stand: the
  walk driver's final summary token
  reads WALK13, furniture carried
  unchanged since Round 19 and named
  here as in Rounds 19 and 20, its cells
  and counts this round's own; and the
  battery driver's summary token stands
  updated to this round's number, its
  docstring and bank renamed with the
  round. The slice and the closure were
  staged in separate directories, the
  slice carrying VL-141 to VL-145 in
  CONTRACT.md alone and the closure
  adding this item and the README line
  at 21 of 100, the README change
  standing in the closure commit only.
- VL-147. The read-back of Round 21's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 34 stands closed and
  merged: its body stands at 16,930
  characters, the lane's body with its
  hosted line, its step 0 section, and
  the CodeRabbit release-notes appendix
  and the owner completion record
  appended after it, its head
  e15293ceafd122219ed11f8b8a64d92194a17a0d
  over the base
  286c0ef56590372fd7310222025026451bc2b733,
  and it landed through the merge queue
  as squash commit
  5f226b024be1480082d4bce2e29888923dbecc80,
  single parent
  286c0ef56590372fd7310222025026451bc2b733,
  the landing tree
  c6f95051fd775ef1d7db730022ea46965e320ff6
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  5f226b02 on a fresh read, its tree at
  c6f95051 over twenty-one blobs, not
  truncated), and the README read on
  main at 21 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7075137465 in its landed
  form (113 characters, merged
  successfully), the CodeRabbit shell
  7075139442 cited in drift-aware form
  (11,118 characters at this hand, edit
  stamp 2026-11-22T06:32:43Z; the Round
  21 lane read 11,116 characters in its
  finished form, the edit stamp
  2026-11-22T06:09:04Z standing in its
  step 0 section, the shell's
  two-character re-render drift standing
  between the reads and the re-render
  carrying the later stamp, and the
  owner's hand reading the same 11,118
  characters with the same later stamp
  in the completion record; created
  2026-11-22T06:07:17Z, Run ID
  b578cd00-304f-4e84-af4c-6c34e89b1735,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head e15293ce,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline comments;
  the shell read whole at this round's
  hand), the merge comment 7075387211
  (12 characters), and the landing
  comment 7076013691 (3,212 characters,
  2026-11-22T07:36:22Z) carrying this
  round's charges; nothing stands after
  the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with one Minor,
  standards APPROVE with one Minor, and
  red team APPROVE with one Minor and
  one carried limitation, no Blocker, no
  Major, and no defect in the round at
  any grade on any axis. Its account
  stands contracted here: the spec Minor
  is a narrative timestamp slip the
  owner corrected in the record itself,
  the body's Disclosures and VL-146
  placing the pin recovery's first dead
  attempt in the same restart that
  killed hold 1, where the lane's own
  record shows the recovery launched at
  20:45:26Z, after the 20:24Z restart
  that killed hold 1, and died in the
  restart at about 22:24Z, VL-146's own
  weather paragraph placing the recovery
  after that 22:24Z restart; the attempt
  stands correctly counted as a dead
  recovery attempt in both texts, the
  census sums exactly at nine holds,
  four ran, five dead, and two dead
  recovery attempts outside the census,
  and no figure rests on the sentence;
  the corrected placement stands
  contracted in this item in one line:
  the pin recovery's first dead attempt
  launched at 20:45:26Z, after the
  20:24Z restart, and died in the
  restart at about 22:24Z. The standards
  Minor is a verification limit on the
  step 0 appendix, the body's claim that
  the CodeRabbit appendix stands
  preserved byte for byte not
  re-verifiable at the standards hand,
  the shell's current form no longer
  carrying the appendix after its
  re-render and the lane's banked shell
  copies standing at zero bytes under
  the write fault, the claim resting on
  the lane's read and corroborated by
  the counts, the body's appendix at
  exactly the lane record's 450
  characters and the body at exactly the
  PATCH-verified 11,978 characters,
  nothing else in the round depending on
  it; this round banks the shell's full
  text at its own step 0 under the
  temp-write-verify discipline, as
  charged, so the preservation claim
  stands re-verifiable at review hands.
  The red team's Minor is a completeness
  observation in VL-144's own terms, the
  suggestion ladder stated whole (keep
  at no regressed measures, rerun at
  one, kill at two or more) while no
  cell regressed two measures, so the
  kill rung stood unexercised, and a
  zero-baseline measure in the higher
  direction stood likewise uncelled, N22
  to N24 covering the zero baseline only
  for the lower direction, the surface
  delivering exactly its contracted
  twenty-six cells; this round's VL-150
  part (a) is the charged extension and
  cells both shapes. VL-144's fresh
  surface, the named margins, ran clean
  on its first run, twenty-six of
  twenty-six with no amendment, the
  fourth consecutive first-run-clean
  surface on this repo, and the
  first-run claim was attacked against
  the disk at the red team's hand and
  held, exactly one driver composed
  before the run, one bank and one run
  log byte-identical at twenty-six PASS
  lines, one work directory, the
  append-mode bank undoubled; the VL-119
  gate form discharged a fifth round
  running, in the substance at both
  trees, the full test result lines
  banked with no tail window anywhere in
  the chain and a working per-file
  codepoint scan as the run of record,
  the split of three library,
  twenty-four judgment, and twenty
  registry read from each tree's own
  log. And the red team's carried
  limitation stands stated as it stands,
  a limitation, not a defect: its
  run-side reproduction stands unrun at
  its hand, the chain it built for the
  reproduction, a fresh build from the
  blob-verified base checkout under the
  slot with drivers re-run from verified
  copies, probe-gating on the write
  fault and the gate never opening, 110
  attempts, one every thirty seconds
  from 2026-11-22T06:36Z to 07:31Z,
  every one refused, the probe file at
  zero bytes at the last attempt; those
  legs rest on the lane's banks, which
  the red team read whole and matched
  cell for cell, and on everything it
  verified firsthand at full strength,
  all three trees listed live at
  twenty-one of twenty-one against its
  own manifest, both gate logs
  re-derived, the hold census checked
  line for line against the four status
  files, the pin log re-summed at 241
  across 19 suites with the golden
  constant standing in source, the fake
  judge md5 re-derived at its hand, the
  chain binary hashed at its hand to the
  claimed sha256, and the margins driver
  read whole with its arms anchored to
  the contracted constants on each arm's
  own registry; and the standards axis's
  one ungraded observation, the shell's
  passed Title check remarking the
  title's density while passing, stands
  on the review record, the check passed
  and the remark non-actionable. This
  round's gate (VL-149) stands in the
  same form at both trees, the
  discharged VL-119 form carried forward
  as the standing form.
- VL-148. The standing regression stands
  re-run whole in its contracted forms
  against a chain binary built fresh
  from the blob-verified base checkout,
  sha256
  a8aa9dbb2090e4e0d78441eedb612a749f0dde02b9303dbfc2b902afdf311d96.
  The judgment battery stands at
  twenty-five of twenty-five, with C17
  to C22 at VL-100's dispositions: C17
  the judge unavailable naming an
  unpinned model, C18 the choice answer
  malformed, C19 the evidence answer
  malformed, C20 the risk answer
  malformed, C21 the bounds holding at
  scores 10 and 0, and C22 the risk
  answer at 105 malformed, the fake
  judge md5 standing at
  a79ded49ae7285fd2d7b028c8b47dd7f,
  re-derived at the instrument copy and
  at the run. The walk stands at ten of
  ten. The flighttape pin stands
  re-verified live: commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at fifty blobs, the pin suite at 241
  tests across 19 suites with none
  failed, summed from the suite's own
  result lines at this round's chain
  run, and the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing in source. The oracle stands
  re-verified: commit
  d62e5dc9547295f9ea7980f8298d905767b22f14,
  tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  at twenty-four blobs, its blobs
  recomputed at the census with none
  bad. The census at the chain run
  stands ALL OK: the main head and tree,
  the twenty-one blobs recomputed
  locally, the base tree's diff against
  Round 21's base touching exactly
  CONTRACT.md and README.md, and item 3
  standing in its three homes.
- VL-149. The local gate stands
  discharged in the VL-119 form at the
  base tree and at the head tree, the
  full cargo test result lines banked at
  both trees with no tail window
  anywhere. At the base tree the test
  lines read 47 passed in the split of 3
  library, 24 judgment, and 20 registry,
  read from the base tree's own log,
  with clippy exit 0, fmt exit 0, and
  the per-file codepoint scan as the run
  of record, its planted control holding
  both dashes at 1 and 1, its totals at
  0 and 0, and scan exit 0. At the head
  tree the same form stands: 47 passed
  in the same split of 3 library, 24
  judgment, and 20 registry, read from
  the head tree's own log, clippy exit
  0, fmt exit 0, and the scan in the
  same form, control 1 and 1, totals 0
  and 0, scan exit 0. Every hold of the
  round stands counted and named here:
  hold 1, the chain hold, waited from
  17:24:09Z, acquired the slot at
  17:27:08Z, and ran to completion at
  17:30:08Z on 2026-11-22, carrying the
  base verification, the base build, the
  base gate block, the census, and the
  pin suite, every step exit 0; hold 2,
  the head gate hold, ran against the
  staged closure tree in the same form.
  No figure in this item rests on a dead
  hold; no hold of the round died.
- VL-150. The fresh surface work stands
  in two parts, each proposed in the
  lane record before running and each
  run clean on its first run with no
  derivation amendment. Part (a) extends
  VL-144's named margins over the two
  shapes Round 21 left uncelled,
  fourteen cells, fourteen of fourteen
  in agreement between the port and the
  oracle, none failed and none blocked.
  The suggestion ladder's kill rung
  stands exercised: exp-k-0001, opened
  at margin 20 over baselines cost 100
  and rescue 80, rested at cost 130 and
  rescue 60, adverse moves of 30 percent
  and 25 percent, both past the margin,
  its close kill resting on lines 1 and
  2 with both measures regressed and the
  suggestion kill, the close agreeing;
  and exp-m-0002, opened with no margin
  so the default 10 stood, rested at
  cost 120 and rescue 60, its close keep
  contested against the same suggested
  kill. The zero-baseline measure in the
  higher direction stands celled twice:
  exp-z-0003, opened at margin 90 over
  baselines cost 100 and rescue 0,
  rested at cost 100 and rescue 5, no
  measure regressed and the suggestion
  keep; and exp-w-0004, opened at margin
  10 with the rescue entry carrying its
  own margin of 1, rested at cost 105
  and rescue 500, no measure regressed,
  the zero baseline leaving the margin
  unconsulted on the rescue measure
  exactly as on exp-z-0003. One
  correction to the round's seating text
  stands recorded in the proposal before
  running: the seating text supposed the
  higher-direction zero-baseline case
  regresses when its rested value is
  adverse-positive, and both shipped
  sources, read whole at the rule, show
  the opposite, the port's zero branch
  returning a regression only for the
  lower direction at a positive value
  and the oracle's higher branch
  clearing its flag at a zero baseline,
  so a zero-baseline measure in the
  higher direction never regresses; the
  cells were derived and run on the
  shipped rule, and both arms agreed on
  every cell. Part (b) takes a fresh
  twin surface Rounds 14 through 21 did
  not measure, the out-of-order clock,
  seventeen cells, seventeen of
  seventeen in agreement, none failed
  and none blocked. Its derivation
  stands read from both sources before
  running: neither source carries a
  temporal-order defect, the only
  timestamp defect in either being the
  malformed at, rests_on stands
  positional, and the VL-9 arithmetic
  never reads the at field, so a
  registry whose clocks run backward
  stands judged by position alone. The
  cells bear it out: exp-t-0001, its
  result dated a week before its open
  and its close dated before both,
  closed keep resting on lines 1 and 2
  with no measure regressed and the
  suggestion keep; exp-u-0002, its
  result dated a year after its open and
  its close dated between the two
  clocks, closed rerun with cost
  regressed on its numbers alone;
  exp-v-0003 and exp-w-0004 interleaved,
  each close resting on its own open and
  result lines, exp-v's keep agreeing
  and exp-w's kill contested against a
  suggested keep; and exp-x-0005, its
  open, result, and close at one
  identical instant, closed keep with no
  measure regressed.
- VL-151. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-152. Round 22 stands closed. The
  README count stands at 22 of 100 in
  this closure commit only, the slice
  carrying VL-147 to VL-151 in
  CONTRACT.md alone and the closure
  adding this item and the README line,
  the two staged in separate
  directories, CONTRACT append-only
  throughout, and the tree diff from the
  base to the head touching exactly
  CONTRACT.md and README.md. The round's
  whole hold census stands here: two
  slot holds, both ran, hold 1 the chain
  hold of VL-149 and hold 2 the head
  gate hold, and no hold died. The
  round's weather stands carried by
  name: the write fault's fsync wave
  stood shut from seating at about
  07:36Z until 17:21:22Z on 2026-11-22,
  about nine and three-quarter hours,
  across four runtime restarts at about
  07:52Z, 09:33Z, 11:34Z, and 17:19Z;
  three openers probed the wave in the
  sitting's background, the first
  spending its 400 attempts by 14:37Z,
  the second killed by the 17:19Z
  restart, and the third finding the
  wave open at its launch and banking
  the read-back, both extractions, and
  the instrument copies in one pass,
  every step exit 0; no figure in the
  round rests on a dead opener, a dead
  hold, or a refused probe. The
  instrument disclosures stand as in
  Rounds 19 to 21: the battery and the
  walk were renamed only in round
  furniture for this round, and the walk
  driver's summary token reads WALK13,
  furniture carried since Round 19.
- VL-153. The read-back of Round 22's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 36 stands closed and
  merged: its body stands at 12,247
  characters at this hand, the lane's
  body with its step 0 section and the
  CodeRabbit release-notes appendix and
  the owner completion record appended
  after it, its head
  e5096a1d00f6c787ee1e13d26505b4a3337ae974
  over the base
  5f226b024be1480082d4bce2e29888923dbecc80,
  and it landed through the merge queue
  as squash commit
  1c04e7c59495b48b59e19c92eb0d71b0c4fdf06c,
  single parent
  5f226b024be1480082d4bce2e29888923dbecc80,
  the landing tree
  b045f04a6c4aaff9158df6c482f83fc407de9628
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  1c04e7c5 on a fresh read, its tree at
  b045f04a over twenty-one blobs, not
  truncated), and the README read on
  main at 22 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7083160653 in its landed
  form (113 characters, created
  2026-11-22T17:51:44Z, last edited
  2026-11-22T19:16:54Z), the CodeRabbit
  shell 7083162914 cited in drift-aware
  form (11,092 characters at this hand,
  edit stamp 2026-11-22T19:09:10Z; the
  Round 22 lane read 11,090 characters
  in its finished form, the edit stamp
  2026-11-22T17:54:05Z standing in its
  step 0 section, the shell's
  two-character re-render drift standing
  between the reads and the re-render
  carrying the later stamp, and the
  owner's hand reading the same 11,092
  characters with the same later stamp
  in the completion record; created
  2026-11-22T17:51:54Z, Run ID
  95a1b602-cbf8-46ac-ac4d-fcc2647e8cfe,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head e5096a1d,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline comments;
  the shell read whole at this round's
  hand), the merge comment 7084152360
  (12 characters, created
  2026-11-22T19:09:24Z), and the landing
  comment 7085035533 (2,928 characters,
  2026-11-22T20:12:35Z) carrying this
  round's charges; nothing stands after
  the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with one Minor,
  standards APPROVE with no findings,
  and red team APPROVE with one carried
  limitation, no Blocker and no Major at
  any hand. Its account stands
  contracted here: the spec Minor is
  disclosure completeness, the shared
  quota's hard stop standing nowhere in
  the body, only in the lane record, the
  stop alone delaying the step 0
  disposition past the shell's finish,
  nothing in the body contradicting it
  and no figure resting on the omission;
  the stop stands contracted in this
  item in one line: the shared quota
  stood at zero at 17:55:47Z on
  2026-11-22, mid-poll, all API work
  stopped with no retries, the step 0
  batch ran after the reset, and the
  merge comment stands created at
  19:09:24Z; and this round states any
  environmental stop that delays its own
  step 0 disposition in its body itself,
  the standing form from this round. The
  standards axis found no findings, and
  the step 0 evidence bank charged by
  Round 21's standards Minor discharged
  in full at its hand: the banked
  shell36.md stands verified at exactly
  11,101 bytes, its content the shell's
  text at the lane's hand, and the
  appendix preservation proven from two
  independent banks, the lane's banked
  shell text and the live body, not the
  lane's read alone; this round banks
  its own shell's full text at step 0
  time under the same discipline, as a
  standing charge. The round's center
  stands contracted with it: VL-150 part
  (a) exercised the two shapes Round 21
  left uncelled, the suggestion ladder's
  kill rung (a close regressing two
  measures, suggested kill on both arms)
  and the zero-baseline measure in the
  higher direction (celled twice, the
  margin unconsulted), and the lane's
  correction to the seating text's
  parenthetical stands sustained at
  three hands against both shipped
  sources, stated here as the shipped
  rule: at a zero baseline a measure
  regresses exactly when its direction
  is lower and its rested value is
  positive, the margin unconsulted, and
  kill stands at two or more regressed;
  the seating text was the error and the
  cells ran on the shipped rule. Part
  (b), the out-of-order clock, agreed 17
  of 17 on its first run, rests_on
  positional in both sources and no
  temporal-order defect in either, the
  fifth consecutive round on this repo
  whose fresh surface work needed no
  amendment. And the red team's carried
  limitation stands stated as it stands,
  a limitation, not a defect: its own
  battery and walk re-run stands unrun
  at its hand, its fresh chain build
  queued on the build slot behind nine
  standing waiters from 19:16Z and
  killed at finalization at 20:10Z, the
  write fault independently refusing the
  registry writes the re-run needed, so
  those figures rest on the lane's banks
  read whole and matched line for line,
  and on everything it verified
  firsthand, the census's live claims,
  the pin and oracle manifests, the fake
  judge md5 re-derived at its hand, and
  the chain binary hashed at its hand to
  the claimed sha256; its two extra
  cases stand composed in its scratch
  and unrun, refused by the same shut
  fault, a fifteenth margins case and an
  eighteenth clock case, and this
  round's VL-156 part (a) is the charged
  run of both. The ungraded observations
  stand (hold 2 of Round 22 named by
  role and fate in VL-149, its times in
  the body and the status file, which
  agree; the appendix measured 496
  characters from its marker against the
  record's 498, the two separator
  newlines at the boundary, the
  preservation itself exact; the Title's
  length, its check passed and the
  remark non-actionable). This round's
  gate (VL-155) stands in the same form
  at both trees, the discharged VL-119
  form carried forward as the standing
  form.
- VL-154. The standing regression stands
  re-run whole in its contracted forms
  against a chain binary built fresh
  from the blob-verified base checkout,
  sha256
  898d79be5e8b09848507de0ea7c9ba92a6cff5203d7772793fc622a521bdf110.
  The judgment battery stands at
  twenty-five of twenty-five, with C17
  to C22 at VL-100's dispositions: C17
  the judge unavailable naming an
  unpinned model, C18 the choice answer
  malformed, C19 the evidence answer
  malformed, C20 the risk answer
  malformed, C21 the bounds holding at
  scores 10 and 0, and C22 the risk
  answer at 105 malformed, the fake
  judge md5 standing at
  a79ded49ae7285fd2d7b028c8b47dd7f,
  re-derived at the instrument copy and
  at the run. The walk stands at ten of
  ten. The flighttape pin stands
  re-verified live: commit
  74a83298aeb31c82e62b9fb0c613f5bc8600799e,
  tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  at fifty blobs, the pin suite at 241
  tests across 19 suites with none
  failed, summed from the suite's own
  result lines at this round's chain
  run, and the golden digest
  cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
  standing in source. The oracle stands
  re-verified: commit
  d62e5dc9547295f9ea7980f8298d905767b22f14,
  tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  at twenty-four blobs, its blobs
  recomputed at the census with none
  bad. The census at the chain run
  stands ALL OK: the main head and tree,
  the twenty-one blobs recomputed
  locally, the base tree's diff against
  Round 22's base touching exactly
  CONTRACT.md and README.md, and item 3
  standing in its three homes.
- VL-155. The local gate stands
  discharged in the VL-119 form at the
  base tree and at the head tree, the
  full cargo test result lines banked at
  both trees with no tail window
  anywhere. At the base tree the test
  lines read 47 passed in the split of 3
  library, 24 judgment, and 20 registry,
  read from the base tree's own log,
  with clippy exit 0, fmt exit 0, and
  the per-file codepoint scan as the run
  of record, its planted control holding
  both dashes at 1 and 1, its totals at
  0 and 0, and scan exit 0. At the head
  tree the same form stands: 47 passed
  in the same split of 3 library, 24
  judgment, and 20 registry, read from
  the head tree's own log, clippy exit
  0, fmt exit 0, and the scan in the
  same form, control 1 and 1, totals 0
  and 0, scan exit 0. Every hold of the
  round stands counted and named here:
  hold 1, the chain hold, waited from
  21:06:35Z, acquired the slot at
  21:18:10Z, and ran to completion at
  21:20:48Z on 2026-11-22, carrying the
  base verification, the base build, the
  base gate block, the census, and the
  pin suite, every step exit 0; hold 2,
  the head gate's first launch, waited
  from 23:15:33Z, acquired the slot at
  01:10:34Z on 2026-11-23, and died in a
  runtime restart at about 01:11Z during
  its probe loop, before the gate block
  ran, its gate log never created; hold
  3, the head gate's second launch,
  waited from 01:19:24Z, acquired the
  slot at 02:05:15Z, and died on its
  probe gate, the write fault re-shut,
  after 240 refused attempts, the gate
  block never running; hold 4, the head
  gate's third launch, waited on the
  wave in the same sitting and ran
  against the staged closure tree in the
  same form. No figure in this item
  rests on a dead hold.
- VL-156. The fresh surface work stands
  in two parts, each proposed in the
  lane record before running and each
  run clean on its first run with no
  derivation amendment. Part (a) runs
  the Round 22 red team's two composed
  cases by name, the driver copied byte
  for byte into this lane's scratch (its
  md5 5d1a65f7dfb9d8b2a8275f2b54fcc9b0
  equal to the original's at the copy,
  its work roots standing under this
  lane's scratch) and run against the
  chain binary and the oracle, ten cells
  in all, ten of ten in agreement, none
  failed and none blocked, each case
  recorded here as an extension case of
  its surface. The fifteenth margins
  case agrees: exp-q-0001, opened over
  baselines cost 100 and rescue 0 with
  no margin named so the default ten
  stood, rested at cost 130 and rescue
  500, its close rerun resting on lines
  1 and 2 with regressed naming cost
  alone and the suggestion rerun on both
  arms, rerun and not kill under the
  sustained shipped rule, the rescue
  measure resting at 500 on a zero
  baseline in the higher direction and
  never regressing. The eighteenth clock
  case agrees: exp-b-0002, opened at
  line 2 at an earlier clock than
  exp-a-0001 at line 1, its result at
  line 3 at a clock before both opens
  and its close at line 4 at the
  earliest clock in the registry,
  resting on lines 2 and 3 with
  regressed empty and the suggestion
  keep on both arms; and exp-a-0001, its
  result landing at line 5 after that
  close and at a clock before exp-b's
  open, its close resting on lines 1 and
  5 with regressed empty and the
  suggestion keep on both arms, the
  close order reversed against both
  clock order and position order and
  rests_on staying positional. Part (b)
  takes a fresh twin surface Rounds 14
  through 22 did not measure, the
  standing experiment's refusals,
  sixteen cells, sixteen of sixteen in
  agreement between the port and the
  oracle, none failed and none blocked.
  Its derivation stands read from both
  sources before running: an opening
  offered for an id already standing in
  the folded states refuses with the
  line that the experiment is already
  open, the fold retaining closed
  experiments so a closed id refuses the
  same way; a close offered for an id
  already closed refuses with the line
  that the experiment is already closed;
  both refusals raise before any append,
  each CLI printing the error line to
  stderr and exiting 1 with stdout
  empty, so a refused write leaves the
  registry byte-unchanged. The cells
  bear it out: exp-a-0001's duplicate
  openings, the identical invocation and
  a varied one at margin 5 over
  different baselines, each refused at
  exit 1 with the contracted line
  byte-equal on both arms and the
  registry standing at one line; its
  result then landing on the same
  registry, and its close keep agreeing,
  resting on lines 1 and 2 with no
  measure regressed; its re-opening
  after that close refused as already
  open at three lines, a closed id
  staying taken; and its second closes
  refused as already closed at three
  lines, at a kill verdict and at the
  landed keep verdict alike.
  Exp-b-0002's duplicate opening refused
  at four lines; its result landing, its
  close rerun agreeing over a 30 percent
  adverse cost move, resting on lines 4
  and 5 with cost regressed; and its
  re-opening after the rerun close
  refused as already open at six lines,
  a rerun close not freeing the id. Show
  and list read back the registry the
  refusals left untouched, both
  experiments closed with their verdicts
  and suggestions standing, and the
  final registry stands at six lines on
  both arms after a closing check.
- VL-157. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-158. Round 23 stands closed. The
  README count stands at 23 of 100 in
  this closure commit only, the slice
  carrying VL-153 to VL-157 in
  CONTRACT.md alone and the closure
  adding this item and the README line,
  the two staged in separate
  directories, CONTRACT append-only
  throughout, and the tree diff from the
  base to the head touching exactly
  CONTRACT.md and README.md. The round's
  whole hold census stands here: four
  slot holds, hold 1 the chain hold of
  VL-155 ran, hold 2 the head gate's
  first launch acquired the slot and
  died in a runtime restart during its
  probe loop before the gate block ran,
  hold 3 the head gate's second launch
  acquired the slot and died on its
  probe gate after 240 refused attempts
  on the re-shut fault, and hold 4 the
  head gate's third launch ran; no
  figure in the round rests on a dead
  hold. The round's weather stands
  carried by name: the write fault's
  fsync wave stood shut from seating at
  about 20:13Z until 21:02:19Z on
  2026-11-22, about forty-nine minutes
  across one runtime restart at about
  21:01Z, opened for the read-back, both
  extractions, the instrument copies,
  and the chain, then re-shut between
  the chain's completion at 21:20:48Z
  and the lane's refused probe at about
  21:39Z and stood shut until 23:11:10Z,
  about an hour and a half from that
  probe, across one further runtime
  restart; two openers probed the first
  shut in the sitting's background, the
  first killed by the 21:01Z restart,
  the second finding the wave open at
  its launch and banking the read-back,
  both extractions, and the instrument
  copies in one pass, every step exit 0;
  the four measurements' first launches
  died at exit 120 on the re-shut fault,
  their run logs at zero bytes, before
  any cell ran; and two measurement
  waiters probed the second shut, the
  first killed by the further restart,
  the second finding the wave open at
  its launch and running all four
  measurements clean in one pass, every
  exit 0; and a third runtime restart at
  about 01:11Z on 2026-11-23 killed the
  head gate's first launch after it
  acquired the slot, as VL-155 records;
  and the fault re-shut a third time
  before the head gate's second launch
  probed, the lane's own probe refusing
  at 02:14:44Z, and stood shut through
  that launch's 240 refused probe
  attempts, the head gate's third launch
  waiting on the wave in the sitting and
  running when it opened. No figure in
  the round rests on a dead opener, a
  dead waiter, a dead launch, a dead
  hold, or a refused probe. The
  instrument disclosures stand as in
  Rounds 19 to 22: the battery and the
  walk were renamed only in round
  furniture for this round, and the walk
  driver's summary token reads WALK13,
  furniture carried since Round 19; the
  extra-cases driver of part (a) is the
  Round 22 red team's own, copied byte
  for byte, and the refusals driver of
  part (b) is new this round.
- VL-159. The read-back of Round 23's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 38 stands closed and
  merged: its body stands at 13,475
  characters at this hand, the lane's
  body of 9,989 characters with its step
  0 section and the CodeRabbit
  release-notes appendix and the owner
  completion record appended after it,
  its head
  e8bec94d97580721f4ba235636b822e799dcb374
  over the base
  1c04e7c59495b48b59e19c92eb0d71b0c4fdf06c,
  and it landed through the merge queue
  as squash commit
  752a4b917bd75ac0bc2700c31d6169e426424c25,
  single parent
  1c04e7c59495b48b59e19c92eb0d71b0c4fdf06c,
  the landing tree
  8ec7e0466db1fafca673f504ca24673eb5501746
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  752a4b91 on a fresh read, its tree at
  8ec7e046 over twenty-one blobs, not
  truncated), and the README read on
  main at 23 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7091454260 in its landed
  form (113 characters, created
  2026-11-23T04:17:27Z, last edited
  2026-11-23T05:35:04Z), the CodeRabbit
  shell 7091457217 cited in drift-aware
  form (10,965 characters at this hand,
  edit stamp 2026-11-23T05:28:00Z; the
  Round 23 lane read 10,961 characters
  in its finished form, created
  2026-11-23T04:17:39Z, edited
  2026-11-23T04:19:20Z, the four
  characters of re-render drift standing
  between the reads; Run ID
  3872a2c5-4a7f-45ea-9e14-0ed8bfb1529e,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head e8bec94d,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline comments;
  the shell read whole at this round's
  hand), the merge comment 7092431689
  (12 characters, created
  2026-11-23T05:28:04Z), and the landing
  comment 7092925928 (2,868 characters,
  2026-11-23T06:00:42Z) carrying this
  round's charges; nothing stands after
  the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with one Minor,
  standards APPROVE with no findings,
  and red team APPROVE with one carried
  limitation, no Blocker and no Major at
  any hand. Its account stands
  contracted here: the spec Minor is
  read-back completeness, VL-153
  omitting Round 22's weather duration
  figure and opener count as stated
  figures, no Round 23 figure resting on
  the omission, and this round contracts
  them in one line: Round 22's fault
  stood shut for about nine and
  three-quarter hours from seating to
  17:21:22Z on 2026-11-22, three openers
  probed it, and the third found the
  wave open. The standards axis found no
  findings, and its two carried
  disciplines stand discharged at their
  hands: the step 0 evidence bank (the
  banked shell38.md stands verified at
  10,972 bytes in the Round 23 lane's
  scratch, its content the shell's text
  at that lane's hand), and the in-body
  delay statement in its first round
  (the fourth shut refused the bank's
  first write at about 04:24Z and the
  bank waiter died in the restart at
  about 05:26Z, both stated in the body
  itself), and the quota stop at branch
  creation (the shared quota at zero
  until the reset at
  2026-11-23T04:08:24Z) stands stated in
  the body's Disclosures. The push
  incident stands as it stands,
  disclosed in the body in full and
  verified clean at three hands: the
  branch chain stands exactly the clean
  pair, the slice
  c1ad1f892d13c750c5335f781f888455ac71565e
  then the closure e8bec94d, the two
  stray commits from the misread first
  push dangling and reachable from no
  ref among the repository's
  fifty-seven, and the mixed tree
  standing in no commit in the chain.
  The round's center stands contracted
  with it: the red team's composed cases
  stand discharged by execution, the
  fifteenth margins case suggesting
  rerun, not kill, the margin's band
  derived in the record from the case's
  own figures, and the eighteenth clock
  case agreeing; and the standing
  experiment's refusals agreed 16 of 16,
  each refusal's stderr asserted
  byte-equal to its contracted line on
  both arms, the registry and stdout
  comparisons carrying the standing cut
  from the first close. And the red
  team's carried limitation stands
  stated as it stands, a limitation, not
  a defect: its own battery and walk
  re-run, one composed-case re-run, and
  the seventeenth refusal case stand
  unrun at its hand under a fault shut
  through its window, ten fsync probes
  refused between 2026-11-23T05:55:37Z
  and 05:58:42Z, so those figures rest
  on the lane's banks read whole and on
  everything it verified firsthand; the
  seventeenth case stands contracted as
  composed but unrun, a second close on
  the rerun-closed experiment exp-b-0002
  expecting exit 1 with the
  already-closed line on both arms, and
  this round's VL-162 part (a) is the
  charged run of it. The ungraded
  observations stand (the body's
  nothing-to-dismiss standing against
  the gate report's one Title dismissal,
  the dismissal recorded at the owner's
  hand; the Title's length). This
  round's gate (VL-161) stands in the
  same form at both trees, the
  discharged VL-119 form carried forward
  as the standing form.
- VL-160. The standing regression stands
  run in the VL-119 form against a chain
  binary built fresh from the
  blob-verified base checkout (sha256
  1b42d6af0f7c5795c5c5ea32c3722f73713095414f8ffcde5df108df04f4a237).
  The judgment battery stands at 25 of
  25 in its amended second bank: the
  first run banked 5 of 25, its twenty
  judge-consulting cells failing because
  the opener's furniture rename missed
  the fake judge reference in the
  battery driver, which pointed the
  judge command at a file that does not
  exist under that name; the amendment
  completed the rename to the banked
  fake judge (md5
  a79ded49ae7285fd2d7b028c8b47dd7f,
  byte-identical to its Round 23 source)
  and the re-run banked under new work
  and bank names at 25 of 25, the first
  bank standing unoverwritten, with
  cells C17 to C22 at VL-100's
  dispositions (the unavailable cells
  closing with their contracted reasons,
  the judge's answer naming an unpinned
  model, its choice answer malformed,
  its evidence answer malformed, its
  risk answer malformed at C20 and again
  at C22, and the bounds cell scoring 10
  and 0). The walk stands at 10 of 10,
  its bank's summary keeping the
  instrument's standing label. The pin
  stands re-verified: flighttape at
  74a83298, its tree
  afd4d7d13520411e003762548fc79082a1b9cf6e
  over 50 blobs recomputed at the
  census, its suite at 241 across 19
  suites from the pin log's own result
  lines, and the golden digest standing
  in source. The oracle stands
  re-verified: d62e5dc, its tree
  74394802a20e03fd9f3e2c5cf781c4ef8936df21
  over 24 blobs, the extraction verified
  blob by blob at the opener.
- VL-161. The local gate stands run at
  the base and head trees in the VL-119
  form, the full test lines banked in
  each tree's own gate log. At the base
  tree: cargo test 47 in the 3 library,
  24 judgment, and 20 registry split,
  test exit 0, clippy exit 0, and fmt
  exit 0 from gate_base.log; the
  codepoint scan's chain invocation
  failed on a directory argument after
  its control read 1 and 1, and the scan
  stands banked per-file at the lane's
  hand over the base checkout's fifteen
  scanned files (control 1 and 1, totals
  0 and 0, exit 0,
  scan_base_supplement.log), the failed
  chain invocation standing disclosed,
  not counted as evidence. At the head
  tree: cargo test 47 in the same 3 plus
  24 plus 20 split, test, clippy, and
  fmt exits 0, and the scan at control 1
  and 1 with totals 0 and 0 and exit 0,
  from gate_head.log, its scan
  invocation amended to the expanded
  file list before the run. The round's
  holds stand counted at three: chain
  launch 1's hold (queued on the slot
  from 07:37Z behind another lane's
  hold, acquired 08:17:52Z, then probing
  the re-shut wave under the slot until
  the restart killed it at about 09:24Z,
  no stage run), chain launch 3's hold
  (the whole base gate, census, and pin
  suite at exit 0), and the head gate's
  hold. The dead launches stand counted
  and never counted as evidence: the
  first Round 24 lane itself, ended
  without a report with its work banked;
  opener launch 1 and sequencer launch
  1, the first lane's waiters, dead on
  the shut wave between 07:29Z and
  07:35Z with no stage run; chain launch
  1, killed as stated; and chain launch
  2, whose verify and census legs banked
  at exit 0 while every cargo leg failed
  at exit 127, cargo not found in the
  post-restart window, its binary hash
  file holding the error line in place
  of a hash.
- VL-162. Fresh surface work stands run
  in two parts, each proposed in the
  lane record before running. Part (a):
  the seventeenth refusal case stands
  run as an extension case of the
  standing experiment's refusals, the
  Round 23 red team's composed case
  discharged by execution: the sixteen
  base cells re-ran whole in fresh
  directories at their Round 23 figures
  and the added cell W17, a second close
  on the rerun-closed experiment
  exp-b-0002 under a rerun verdict,
  agreed on both arms at exit 1, its
  stderr byte-equal to the contracted
  already-closed line, its stdout empty,
  and the registry standing at six lines
  on both arms; the surface stands at 17
  of 17 agree, 0 failed, 0 blocked. The
  driver is the lane's Round 23 refusals
  driver extended by the one cell; the
  red team's own driver copy stands
  zeroed in its scratch, and the lane's
  copy was the byte source, as the
  record states. Part (b): the timestamp
  spellings at the open stand run as a
  fresh twin surface of twenty-four
  cells, T01 to T24, on genuinely
  independent arms in fresh directories,
  each close measured against the
  contracted constants on each arm's own
  registry and the registry and stdout
  cut applied fail-closed from the first
  close: 24 of 24 agree, 0 failed, 0
  blocked. Ten spellings stand accepted
  at both gates and seven refused at
  both, the refused spellings' stderr
  byte-equal to the contracted line
  naming the line's position (lines 2 to
  8), the accepted opens recording the
  at string verbatim on each arm, the
  registry line counts standing at the
  contracted sequence after each cell,
  the close on exp-s-0001 resting at (1,
  11) keep, the close on exp-s-0003
  resting at (2, 13) keep, and the show
  of exp-s-0003 reading its opened at
  verbatim as 2026-04-06T09:00:00+02:00
  on both arms.
- VL-163. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-164. Closure. The README stands at
  24 of 100 in the closure commit only;
  the slice and closure stand staged in
  separate directories; the CONTRACT
  stands append-only by byte-prefix; and
  the tree diff from base to head
  touches exactly CONTRACT.md and
  README.md. The round's weather stands
  carried by name: the first lane ended
  without a report and this completion
  lane adopted its banked work; the
  write wave stood shut in its
  fsync-refusal form from before the
  first lane's seating until 07:35:51Z,
  killing the first lane's two waiters
  between 07:29Z and 07:35Z; it shut
  again across the base chain's first
  hold and a runtime restart killed that
  chain at about 09:24Z; the next chain
  launch lost every cargo leg to a
  missing cargo path in the post-restart
  window; and the battery's first run
  failed on a missed furniture rename,
  amended and re-banked as VL-160
  states. Every figure in VL-159 to
  VL-163 rests on banks verified at this
  lane's hand, and the holds and dead
  launches stand counted in VL-161.
- VL-165. The read-back of Round 24's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 40 stands closed and
  merged: its body stands at 11,655
  characters at this hand, the lane's
  body of 7,806 characters with its step
  0 section, the CodeRabbit
  release-notes appendix, and the owner
  completion record appended after it,
  its head
  d3fcbedef09bb23b6ddddb2dac0e8d1310ec6ed0
  over slice
  2a0a295ff426aef3f8bf76e47f3c7375d038e101
  over the base
  752a4b917bd75ac0bc2700c31d6169e426424c25,
  and it landed through the merge queue
  as squash commit
  ce768a35f124c405f8873e3886d8cb7eebc471ba,
  single parent 752a4b91, the landing
  tree
  60146a0fafd992e284af934d84ec1290ec1e4b7a
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  ce768a35 on a fresh read, its tree at
  60146a0f over twenty-one blobs, not
  truncated), and the README read on
  main at 24 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7098174865 (113 characters,
  created 2026-11-23T10:34:47Z, last
  edited 2026-11-23T10:49:03Z), the
  CodeRabbit shell 7098181144 cited in
  drift-aware form (10,991 characters at
  this hand, created
  2026-11-23T10:35:03Z, last edited
  2026-11-23T10:41:22Z; the Round 24
  lane read 10,989 characters in its
  finished form, created
  2026-11-23T10:35:03Z, edited
  2026-11-23T10:37:47Z, the two
  characters of re-render drift standing
  between the reads; Run ID
  df99a37d-d942-4da2-a825-26430c60d075,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head d3fcbede,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline comments;
  the shell read whole at this round's
  hand), the merge comment 7098336817
  (12 characters, created
  2026-11-23T10:41:31Z), and the landing
  comment 7098548756 (3,294 characters,
  created 2026-11-23T10:49:29Z) carrying
  this round's charges; nothing stands
  after the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with one Minor,
  standards APPROVE with no findings,
  and red team APPROVE with one Minor
  and no carried limitation, every
  charged leg run at its hand including
  a whole re-run of REFUSE24 whose bank
  is byte-identical to the lane's, no
  Blocker and no Major at any hand. Its
  account stands contracted here. The
  two Minors are the same scan
  narrowing, stated as a narrowing:
  Round 24's gate scan covered fifteen
  of the tree's twenty-one files at both
  trees, the six omitted all text
  (.github/workflows/ci.yml, .gitignore,
  Cargo.lock, LICENSE,
  scripts/codepoint_scan.py, and
  tests/fixtures/judgments_synthetic.json),
  and neither VL-161 nor the body stated
  the set was narrower than the standing
  whole-tree set; no figure was false,
  the round's own CONTRACT.md and
  README.md standing scanned at both
  trees, the base tree being Round 23's
  landing tree scanned whole at 0 and 0
  in Round 23, and the red team scanning
  the six omitted files at its own hand,
  each reading 0 and 0, so the totals
  hold tree-wide; this round's gate
  (VL-167) restores whole-tree coverage.
  The round's two-lane handover stands
  as it stands: the first Round 24 lane
  ended without a report with its work
  banked whole under the survival rule
  (the full lane record, VL-159 and
  VL-163 in final form, both surface
  drivers composed), and a completion
  lane adopted the banked work and
  carried the round to the gate; the
  owner's reseat brief told the
  completion lane the first lane's
  detached waiters were dead, the
  completion lane found both alive in
  their probe loops, adopted the live
  pair rather than launching a duplicate
  opener that would have truncated the
  shared status file, and corrected the
  record in the open, the error the
  owner's and the correction the lane's,
  neither hidden in the CONTRACT, the
  body, or the record; the waiters died
  in the restart window between 07:29Z
  and 07:35Z on 2026-11-23 with no stage
  run and stand counted. The battery
  amendment stands contracted: the
  battery's first run banked 5 of 25,
  its twenty failures exactly the
  judge-consulting cells, because the
  opener's furniture rename missed the
  fake judge reference in the battery
  driver; the cause stands recorded in
  the lane record before the re-run, the
  re-run banked under new work and bank
  names at 25 of 25, and the first bank
  stands unoverwritten on disk,
  corroborated at the red team's hand
  three ways. The seventeenth refusal
  case stands discharged by execution:
  REFUSE24 at 17 of 17 agree, the added
  cell a second close on the
  rerun-closed experiment exp-b-0002
  refused at exit 1 with the
  already-closed line byte-equal on both
  arms and the registry at six lines on
  both arms, closing the composed-case
  chain that began with Round 22's pair.
  The timestamp spellings at the open
  stand at 24 of 24 agree on their first
  run, ten spellings accepted at both
  gates and seven refused at both with
  refusal lines byte-equal, the closes
  resting at (1, 11) keep and (2, 13)
  keep, and the show reading
  2026-04-06T09:00:00+02:00 verbatim.
  The red team's twenty-fifth case
  stands contracted as composed and run
  at the red team's hand but standing
  outside the lane's own banks:
  uppercase T with lowercase z alone,
  the timestamp 2026-04-06T09:00:00z,
  refused on both arms at exit 1 with
  the contracted-form line, stdout
  empty, no registry created, the arms
  agreeing; its driver stands banked in
  the red team's scratch, and this
  round's VL-168 part (a) is the charged
  run of it in the lane's own banks. One
  clarification from the red team's hand
  stands contracted for the record's
  unambiguity: PR 40's own preserved
  appendix measures 509 characters
  marker to end marker, and the 432
  figure in VL-159 belongs to PR 38's
  appendix, exactly as that item
  contracts it. The ungraded observation
  stands (the Title's length). This
  round's gate (VL-167) stands in the
  same VL-119 form at both trees with
  whole-tree scan coverage restored, as
  the landing comment charges.
- VL-166. Standing regression, in the
  VL-119 form. Every measurement ran
  against the chain binary built fresh
  in this round from the blob-verified
  base checkout (sha256
  5a97eca877984356bd96ff525b25b090973bc5a29e8b31973fecd15143254519).
  The battery stands at 25 of 25 cells
  passing (battery25_results.txt), on
  its first run this round, with C17 to
  C22 at VL-100's dispositions: the
  unavailable cells closing with their
  contracted reasons, the judge's answer
  naming an unpinned model, its choice
  answer malformed, its evidence answer
  malformed, its risk answer malformed
  at C20 and again at C22, and the
  bounds cell scoring 10 and 0; the fake
  judge stands byte-identical at its
  contracted md5
  a79ded49ae7285fd2d7b028c8b47dd7f. The
  walk stands at 10 of 10 cells passing
  (walk25_results.txt; its bank's
  summary line keeps the instrument's
  standing WALK13 label). The pin stands
  re-verified: flighttape commit
  74a83298, tree afd4d7d1, 50 blobs
  confirmed against the live tree at
  this round's census, its suite at 241
  tests passing across 19 suites in the
  pin re-run's bank (pin_suite2.log and
  pin_summary2.txt; the first pin
  launch's failure and the re-run's own
  dead first launch stand in VL-167),
  and the golden digest cc9c4e01
  standing in the pinned source, read at
  this lane's hand in
  tests/digest_cli.rs. The oracle stands
  re-verified: commit d62e5dc, tree
  74394802, 24 blobs, the census
  recomputing its blobs against the
  saved tree census with none bad.
- VL-167. The local gate at base and
  head trees, in the VL-119 form, with
  whole-tree scan coverage restored, the
  Round 24 Minor's charge discharged. At
  the base, gate_base.log in the chain's
  bank: cargo test 47 tests passing in
  the 3 library, 24 judgment, 20
  registry split, clippy and fmt at exit
  0. At the head, gate_head.log: VERIFY
  exit 0 (21 files, bad none), cargo
  test 47 in the same 3 plus 24 plus 20
  split, TEST, CLIPPY, FMT, and SCAN
  exits all 0. The codepoint scan at
  both trees takes an explicit per-file
  list expanded from the tree listing
  and covers every text file in the
  tree, 21 of 21 files at both trees,
  the gate log printing the scanned
  count against the tree's file count;
  the six files Round 24's scan omitted
  (.github/workflows/ci.yml, .gitignore,
  Cargo.lock, LICENSE,
  scripts/codepoint_scan.py,
  tests/fixtures/judgments_synthetic.json)
  stand in the list by name, and no file
  is excluded. The scan's planted
  control convicted inside each gate log
  (control 1 and 1), totals 0 and 0 at
  both trees. Holds and launches: three
  build-slot holds in all (the base
  chain's hold, the pin re-run's hold,
  the head gate's hold). The pin suite's
  first launch, inside the chain, failed
  at exit 101, rustc refused its output
  writes (Operation not permitted) while
  compiling the pinned test targets, the
  write wave fluttering shut minutes
  after the base gate had passed; it
  stands as a failed run, never
  evidence, its bank unoverwritten. The
  pin re-run's first launch died queued
  on the slot in a runtime restart, no
  stage run and no bank, counted dead
  and relaunched in the same sitting to
  exit 0, the amendment recorded in the
  lane record before the re-run. No
  other launch died or failed.
- VL-168. Fresh surface work in two
  parts, each proposed in the lane
  record before it ran. (a) The Round 24
  red team's twenty-fifth spelling case,
  run by name as an extension case of
  the spellings surface
  (case25ext_results.txt): an open of
  exp-s-0025 offered at
  2026-04-06T09:00:00z, uppercase T with
  lowercase z alone, refused on both
  arms at exit 1 with the line error,
  line 1: at is not an ISO timestamp,
  stderr byte-equal across the arms,
  stdout empty on both, no registry
  created on either arm; the arms agree
  and the case stands at 1 of 1 in this
  lane's own banks, the red team's hand
  and this lane's now reading the same.
  The driver stands adapted from the red
  team's banked copy, its case substance
  unchanged. (b) The timestamp spellings
  at the result and the close, a twin
  surface Rounds 14 through 24 did not
  measure, twenty-two cells on genuinely
  independent arms
  (spellclose25_results.txt), standing
  at 22 of 22 agree, 0 failed, 0
  blocked, on its first run. Four
  experiments open canonically; their
  results and closes then carry the
  spelling classes the spellings surface
  measured only at the open. Accepted at
  the result: the extended offset, the
  fraction with Z, the space separator,
  the fraction with offset, and the
  negative offset with minutes. Accepted
  at the close: the reduced time, the
  naive stamp, the basic offset, and the
  canonical stamp. Refused at the
  result: the date alone and the
  full-day offset. Refused at the close:
  the lowercase t, the thirteenth month,
  and the leap second. Every refusal
  exits 1 on both arms with the single
  contracted line naming its own line
  position (lines 6, 7, 9, 11, and 13
  across the surface), stderr
  byte-equal, stdout empty, the registry
  byte-unchanged on both arms. The
  closes rest at their contracted
  constants on each arm's own registry:
  exp-r-0001 at (1, 6), suggested keep;
  exp-r-0002 at (2, 8), suggested rerun
  with cost regressed; exp-r-0003 at (3,
  10), suggested keep; exp-r-0004 at (4,
  12), suggested rerun with rescue
  regressed. The shows read the spelled
  stamps back verbatim, the offset and
  fraction forms at the results and the
  reduced and basic forms at the closes,
  with rests and verdicts agreeing; the
  list shows the four closed; check
  exits 0 with the registry at thirteen
  lines.
- VL-169. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-170. Closure. README moves to 25 of
  100 in the closure commit only; slice
  and closure staged in separate
  directories (stage_slice,
  stage_closure); CONTRACT append-only
  by byte-prefix; the tree diff base to
  head touches exactly CONTRACT.md and
  README.md. The round's weather, by
  name: the write wave's flutter closed
  the pin suite's first launch at exit
  101, discharged by the re-run banked
  under new names; a runtime restart
  killed the pin re-run's first launch
  while it queued on the build slot for
  about twenty-three minutes, counted
  dead and relaunched in the same
  sitting; no other launch died. No
  launch's failure is counted as
  evidence anywhere in this round's
  items.
- VL-171. The read-back of Round 25's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 42 stands closed and
  merged: its body stands at 11,749
  characters at this hand, the lane's
  body of 8,965 characters with its step
  0 section, the CodeRabbit
  release-notes appendix, and the owner
  completion record appended after it,
  its head
  abe26ee32ecfc10a0cbcce5a8dd62d858c25c35a
  over slice
  a30a714118f6b5e73766cfd554b90b3d88bcbf72
  over the base
  ce768a35f124c405f8873e3886d8cb7eebc471ba,
  and it landed through the merge queue
  as squash commit
  db873240f1e0ae661afa3ac8fa320b27719f5fd2,
  single parent ce768a35, the landing
  tree
  b2b047157166117c7945bf5bc0c778792301a81c
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  db873240 on a fresh read, its tree at
  b2b04715 over twenty-one blobs, not
  truncated), and the README read on
  main at 25 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7099997295 (113 characters,
  created 2026-11-23T11:43:09Z, last
  edited 2026-11-23T11:55:20Z), the
  CodeRabbit shell 7100001477 cited in
  drift-aware form (11,076 characters at
  this hand, created
  2026-11-23T11:43:20Z, last edited
  2026-11-23T11:47:07Z; the Round 25
  lane read 11,074 characters in its
  finished form, created
  2026-11-23T11:43:20Z, edited
  2026-11-23T11:45:11Z, the two
  characters of re-render drift standing
  between the reads; Run ID
  a0c2deab-dbe3-42bc-99c4-657585a8b979,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head abe26ee3,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline comments;
  the shell read whole at this round's
  hand and standing banked whole in the
  Round 25 lane's scratch from its step
  0), the merge comment 7100085832 (12
  characters, created
  2026-11-23T11:47:12Z), and the landing
  comment 7100374157 (3,060 characters,
  created 2026-11-23T12:00:34Z) carrying
  this round's charges; nothing stands
  after the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with no
  findings, standards APPROVE with no
  findings, and red team APPROVE with
  one Minor, a carried limitation on its
  own composed extra case touching no
  claim of the lane's, no Blocker, no
  Major, and no finding against the lane
  at any hand, the cleanest verdict set
  the bench has recorded on this repo.
  Its account stands contracted here.
  The whole-tree scan's restoration
  stands verified at three hands: each
  Round 25 gate log lists exactly the
  tree's twenty-one text files against
  the API listings, the six files Round
  24's scan omitted
  (.github/workflows/ci.yml, .gitignore,
  Cargo.lock, LICENSE,
  scripts/codepoint_scan.py, and
  tests/fixtures/judgments_synthetic.json)
  in the list by name at both trees, the
  planted control convicted inside each
  log, totals 0 and 0, and the red team
  enumerating each tree's text files
  from the API listings and confirming
  the scan's list is exactly that set;
  this round's gate (VL-173) keeps
  whole-tree coverage standing. The
  twenty-fifth spelling case stands in
  the Round 25 lane's own banks:
  CASE25EXT at 1 of 1 agree, uppercase T
  with lowercase z alone at an open
  (2026-04-06T09:00:00z), refused on
  both arms at exit 1 with the
  contracted-form line, stdout empty, no
  registry created, the adaptation of
  the red team's banked driver confined
  to the docstring, the paths, and one
  added byte-constant assertion. The
  spell-close surface stands at 22 of 22
  agree on its first run, the timestamp
  spellings at the result and the close,
  its closes resting in the bank's order
  at (1, 6) keep, (2, 8) rerun with cost
  regressed, (3, 10) keep, and (4, 12)
  rerun with rescue regressed, five
  refusals byte-equal at lines 6, 7, 9,
  11, and 13. The pin chain's weather
  stands contracted: the pin suite's
  first launch failed at exit 101 (rustc
  refused output writes, Operation not
  permitted, the write wave fluttering
  shut minutes after the base gate
  passed), standing as a failed run with
  its bank unoverwritten; the re-run
  charged under an amendment recorded in
  the lane record before it ran, banking
  under new names; the re-run's first
  launch died queued on the build slot
  in a runtime restart (about
  twenty-three minutes, no stage run, no
  bank), counted dead and relaunched in
  the same sitting to exit 0 (241 tests
  across 19 suites). The red team's
  twenty-third case stands contracted as
  composed but unrun: a result at
  2026-04-13T09:00:00z, lowercase z
  alone at a result, screened as failing
  in the Round 25 lane's derivation but
  never celled; ten attempts and two
  direct probes by the red team all
  stopped at the case's canonical open,
  both arms' registry appends refused
  with the wave's EPERM signature (Errno
  1), the arms blocking identically so
  no divergence was observed; its driver
  stands at the Round 25 red team's
  scratch, and this round's VL-174 part
  (a) is the charged run of it in this
  lane's own banks. PR 42's preserved
  appendix stands whole in its body at
  678 characters marker to end marker at
  the standards hand, in the convention
  that gives PR 40's contracted 509. The
  ungraded observations stand (the
  Title's length; a doubled period in
  the body's Verified block).
- VL-172. Standing regression, in the
  VL-119 form. Every measurement ran
  against the chain binary built fresh
  in this round from the blob-verified
  base checkout (sha256
  2fdcfec0588d0556ba9dab5d4ae95169abba665fb8c3f46cfde734035c355f6a).
  The battery stands at 25 of 25 cells
  passing (battery26m_results.txt), with
  C17 to C22 at VL-100's dispositions:
  the unavailable cells closing with
  their contracted reasons, the judge's
  answer naming an unpinned model, its
  choice answer malformed, its evidence
  answer malformed, its risk answer
  malformed at C20 and again at C22, and
  the bounds cell scoring 10 and 0; the
  fake judge stands byte-identical at
  its contracted md5
  a79ded49ae7285fd2d7b028c8b47dd7f. The
  walk stands at 10 of 10 cells passing
  (walk26m_results.txt; its bank's
  summary line keeps the instrument's
  standing WALK13 label). The pin stands
  re-verified: flighttape commit
  74a83298, tree afd4d7d1, 50 blobs
  confirmed against the live tree at
  this round's census, its suite at 241
  tests passing across 19 suites in the
  pin re-run's bank (pin_suite2.log and
  pin_summary2.txt), and the golden
  digest cc9c4e01 standing in the pinned
  source, read at this lane's hand in
  tests/digest_cli.rs. The oracle stands
  re-verified: commit d62e5dc, tree
  74394802, 24 blobs, the census
  recomputing its blobs against the
  saved tree census with none bad. The
  measurement banks' provenance stands
  stated here because it is not the
  standing form: this round was carried
  by two lanes in parallel after the
  first lane's reported death (VL-176),
  and the measurements ran under both.
  The first launches ran concurrently,
  two launches of the same script
  against the same work roots and the
  same append-mode banks, and their
  banks interleaved
  (battery26_results.txt at 50 PASS
  lines for 25 cells, walk26_results.txt
  at 20 for 10, case23ext26_results.txt
  doubled, numforms26_results.txt
  interleaved with the colliding run's
  two FAIL lines at its own late cells);
  those four banks stand unoverwritten
  as the collision's record and no
  figure in this round is taken from
  them. The first lane then ran its
  amended 26c sequence (its launches 3,
  4, and 5 in the lane record: an
  invocation failure creating no bank, a
  battery banked whole with the other
  three drivers failed at their
  invocation, and the walk, the case,
  and the number forms banked clean),
  and the completion lane ran its
  amended re-run (measure26m.sh, single
  launch, renamed drivers with furniture
  renames only, new work roots and new
  bank names, the amendment recorded in
  the lane record before the re-run).
  The two clean bank sets agree at every
  figure; the figures above and in
  VL-174 are taken from the re-run's
  banks (the 26m set), each verified
  whole at this hand (exact PASS counts,
  no duplicated cell, byte-identical to
  its stdout bank where the instrument
  banks its stdout).
- VL-173. The local gate at base and
  head trees, in the VL-119 form, with
  whole-tree scan coverage standing. At
  the base, gate_base.log in the chain's
  bank: cargo test 47 tests passing in
  the 3 library, 24 judgment, 20
  registry split, clippy and fmt at exit
  0. At the head, gate_head.log: VERIFY
  exit 0 (21 files, bad none), cargo
  test 47 in the same 3 plus 24 plus 20
  split, TEST, CLIPPY, FMT, and SCAN
  exits all 0. The codepoint scan at
  both trees takes an explicit per-file
  list expanded from the tree listing
  and covers every text file in the
  tree, 21 of 21 files at both trees,
  the gate log printing the scanned
  count against the tree's file count;
  the six files Round 24's scan omitted
  (.github/workflows/ci.yml, .gitignore,
  Cargo.lock, LICENSE,
  scripts/codepoint_scan.py,
  tests/fixtures/judgments_synthetic.json)
  stand in the list by name, and no file
  is excluded. The scan's planted
  control convicted inside each gate log
  (control 1 and 1), totals 0 and 0 at
  both trees. Holds and launches: the
  round's build-slot holds stand counted
  in the lane record (the base chain's
  hold, the pin re-run's hold, and the
  head gate's holds). The gate script
  launched more than once against the
  same closure tree in the round's
  two-lane running, every launch at exit
  0, the last launch's log standing at
  gate_head.log and an earlier launch's
  log overwritten at that same bank
  name. The opener's first launch probed
  the shut write wave for about
  seventy-seven minutes and died with
  its sitting in a runtime restart, no
  stage run, counted dead; its second
  launch ran the read-back and the
  extractions clean. The pin suite's
  first launch, inside the chain, failed
  at exit 101, five robustness tests
  failing on file operations refused
  (Operation not permitted) while the
  wave fluttered shut minutes after the
  base gate had passed; it stands as a
  failed run, never evidence, its bank
  unoverwritten, and the re-run stands
  charged under the amendment recorded
  in the lane record before it ran. The
  pin re-run's first launch queued on
  the build slot behind another lane's
  gated hold for about one hundred
  minutes (the holder read from the lock
  state, found alive, and waited on,
  nothing signalled) and died queued in
  a runtime restart, never acquiring, no
  stage run and no bank, counted dead.
  The first build lane of this round
  then died on a runtime database error
  at 15:28Z with its record banked
  whole; its relaunched pin re-run was
  found alive by the completion lane,
  adopted and never duplicated, and
  completed under that adoption at exit
  0 (slot acquired 15:37:26Z, pin_suite2
  exit 0 at 15:37:59Z, 241 across 19
  suites, 0 failed). The first
  measurement launch died in the same
  restart against the shut wave, no
  driver run and nothing banked, counted
  dead; the measurement collision, the
  first lane's 26c sequence, and the
  completion lane's amended re-run stand
  in VL-172. No other launch died or
  failed.
- VL-174. Fresh surface work in two
  parts, each proposed in the lane
  record before it ran. (a) The Round 25
  red team's twenty-third spelling case,
  run by name as an extension case of
  the spellings family
  (case23ext26m_results.txt): the open
  of exp-r-0023 at 2026-04-06T09:00:00Z
  exits 0 on both arms, and the result
  at 2026-04-13T09:00:00z, lowercase z
  alone at a result, is refused on both
  arms at exit 1 with the line error,
  line 2: at is not an ISO timestamp,
  stderr byte-equal across the arms,
  stdout empty on both, the registry
  byte-unchanged at one line on both
  arms; the arms agree and the case
  stands at 1 of 1 in this lane's own
  banks, discharged by execution. The
  driver stands adapted from the red
  team's banked case23_rt.py only in its
  docstring, its work root, and its
  bank, its substance, offered stamps,
  invocation form, and judged set the
  red team's own. (b) The measure number
  forms at the open, the result, and the
  close, a twin surface Rounds 14
  through 25 did not measure,
  twenty-four cells on genuinely
  independent arms
  (numforms26m_results.txt), standing at
  24 of 24 agree, 0 failed, 0 blocked.
  Every surface before this one offered
  integer measures in judged order; this
  one cells the forms of the measures
  themselves, the derivation read from
  the shipped source and the candidate
  forms screened through the oracle's
  own parser and defect check before the
  surface ran. Accepted forms stand
  verbatim in each arm's own event at
  its position: integral-float baseline
  measures, exponent-form baselines (1e2
  and 8e1, standing recorded as 100.0
  and 80.0 and read back in those forms
  by the shows), a fraction and an
  integral float at the result (99.5 and
  80.0), integers against a float
  baseline, the judged measures in
  reversed key order, a negative and a
  zero (-5 and 0), a result carrying an
  extra unjudged float measure beside
  the judged pair, and a thirty-digit
  integer cost recorded verbatim on both
  arms. Refused: a NaN baseline measure
  at the open, at line 5 with the
  contracted baseline_measures line, and
  three results at line 9 with the
  contracted measures line (a string
  value, a bool value, an overflowing
  exponent 1e400), every refusal at exit
  1 on both arms with stderr byte-equal
  to the contracted line itself, stdout
  empty, the registry byte-unchanged.
  The closes rest at their contracted
  constants on each arm's own registry:
  (1, 5) keep, (2, 6) keep, (3, 9) keep,
  (4, 8) rerun with rescue regressed,
  and the huge-integer experiment at
  (14, 15) rerun with cost regressed;
  the list shows the five closed and
  check exits 0 with the registry at
  sixteen lines. The surface's first
  running stands inside the collided
  measurement launches of VL-172 (one
  completion at 24 of 24 whose bank
  interleaved, one run failing at its
  late cells against the shared work
  root); the figures here are the
  amended re-run's, reproduced whole
  from a single launch, and the first
  lane's 26c run of the same driver
  reads the same 24 of 24.
- VL-175. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-176. Closure. README moves to 26 of
  100 in the closure commit only; slice
  and closure staged in separate
  directories (stage_slice,
  stage_closure); CONTRACT append-only
  by byte-prefix; the tree diff base to
  head touches exactly CONTRACT.md and
  README.md. The round's weather, by
  name: the write wave stood shut from
  the seating through a runtime restart,
  killing the opener's first launch
  after about seventy-seven minutes of
  probing; the wave's flutter closed the
  pin suite's first launch at exit 101,
  discharged by the amended re-run; a
  runtime restart killed the pin
  re-run's first launch while it queued
  on the build slot for about one
  hundred minutes, counted dead; the
  first build lane died on a runtime
  database error at 15:28Z with its
  record banked whole, and a completion
  lane adopted the round, found the
  relaunched pin re-run alive, and never
  duplicated it; the first lane's
  session in fact kept working after its
  recovery, running its own amended
  measurement sequence and staging a
  draft of its own, and the two lanes'
  measurement launches collided once, as
  VL-172 states; the first lane's draft
  and staging stand preserved in the
  lane scratch under renamed paths, not
  used, for the two named reasons the
  lane record states (its item 3 text
  and its head gate account do not stand
  against the tree and the banks). The
  first measurement launch died in the
  restart against the shut wave. No
  launch's failure is counted as
  evidence anywhere in this round's
  items.
- VL-177. The read-back of Round 26's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 44 stands closed and
  merged: its body stands at 15,208
  characters at this hand, the lane's
  body of 10,978 characters with the
  owner completion record appended after
  it, its head
  2e72003c62c76abaae013164454eaaa7726fae67
  over slice
  426fa3343858e0b69595c09272a979877ca8a009
  over the base
  db873240f1e0ae661afa3ac8fa320b27719f5fd2,
  and it landed through the merge queue
  as squash commit
  6182d23e9086a817de89b2fde307dc320b9f71a3,
  single parent db873240, the landing
  tree
  638f6c176fb09661570a7a5cae5c6b7029fe47b5
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  6182d23e on a fresh read, its tree at
  638f6c17 over twenty-one blobs, not
  truncated), and the README read on
  main at 26 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7109328550 (113 characters,
  created 2026-11-23T17:53:15Z, last
  edited 2026-11-23T18:05:53Z), the
  CodeRabbit shell 7109332978 cited in
  drift-aware form (10,986 characters at
  this hand, created
  2026-11-23T17:53:26Z, last edited
  2026-11-23T17:57:26Z; the Round 26
  lane read 10,984 characters in its
  finished form, created
  2026-11-23T17:53:26Z, edited
  2026-11-23T17:56:30Z, and the owner
  read 10,986 characters at the later
  edit, the two characters of re-render
  drift standing between the lane's read
  and the later reads; Run ID
  742530b0-6cdd-4102-8023-9b57fd7493ad,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head 2e72003c,
  four pre-merge checks passed, no
  actionable comments generated, no
  review object, and no inline comments;
  the shell read whole at this round's
  hand and standing banked whole in the
  Round 26 lane's scratch from its step
  0), the merge comment 7109434148 (12
  characters, created
  2026-11-23T17:57:29Z), and the landing
  comment 7110167746 (3,358 characters,
  created 2026-11-23T18:26:28Z) carrying
  this round's charges; nothing stands
  after the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with two Minors,
  standards APPROVE with two Minors, and
  red team APPROVE with two Minors and
  no carried limitation, every charged
  leg run at its hand, no Blocker and no
  Major at any hand, one Minor
  converging at all three hands. Its
  account stands contracted here. The
  round was carried by two lanes in
  parallel, and the record states it as
  it stands: the first lane's reported
  death at 15:28Z was a runtime database
  error from which it recovered and kept
  working; the completion lane adopted
  its relaunched pin re-run, never
  duplicated it, and the re-run
  completed at exit 0 (241 tests passing
  across 19 suites, 0 failed). The two
  lanes' measurement launches collided
  against shared work roots and
  append-mode banks; the four collided
  banks stand unoverwritten and are
  unusable on their face (the battery
  bank at 50 PASS lines with two
  summaries, the number-forms bank at 46
  PASS lines with two FAIL lines
  interleaved), no figure rests on them,
  and the round's figures stand on the
  completion lane's amended 26m re-run
  with the first lane's amended 26c bank
  set agreeing at every figure, verified
  at three hands. The twenty-third case
  stands discharged by execution:
  CASE23EXT at 1 of 1 agree in the Round
  26 lane's own banks and again at the
  red team's hand. The number-forms
  surface stands at 24 of 24 agree on
  its first run, and the red team's
  composed twenty-fifth case (a raw hex
  literal and an underscore-separated
  measure) was refused on both arms with
  the byte-identical line, the arms
  agreeing on both shapes. The head
  gate's three launches stand with
  launch 3's log standing at
  gate_head.log, the earlier launches'
  logs overwritten at that same bank
  name by the script's design and
  disclosed in VL-173 and the body, all
  three launches timed in
  gate_head_status.txt, no figure
  resting on an overwritten log; this
  round's gate (VL-179) banks each
  launch under its own name. The first
  lane's draft and staging stand
  preserved in the Round 26 lane's
  scratch under renamed paths, not used;
  the rejection of them stands sustained
  on its verified reasons: the draft's
  VL-175 quotes a Rust-port parity
  sentence measuring 155 characters that
  stands nowhere in the tree (and claims
  it at 163), and its VL-173 states the
  head gate's figures flatly at a
  writing before any head gate had run;
  the red team names the provenance the
  evidence supports, all three defects
  summary-carried text asserted without
  a bank behind it. The round's three
  corrections stand carried here, each
  in one line. The rejection entry's
  scan-list limb stands corrected: the
  foreign list stood in the first lane
  draft's first version only and was
  corrected from gate_base.log before
  staging; the preserved draft carries
  the true list; the rejection stands on
  the item 3 text and the head gate
  account. VL-173's closing census
  sentence stands corrected: two
  rebuild-and-gate launches in the head
  gate's own chain died or gave up
  without reaching a gate run, both
  counted in the record. VL-173's
  same-closure-tree phrase stands
  corrected: launches 1 and 2 ran on
  staging run 1's closure, launch 3 on
  run 2's corrected closure.
- VL-178. Standing regression, in the
  VL-119 form. Every measurement ran
  against the chain binary built fresh
  in this round from the blob-verified
  base checkout (sha256
  9092f9d4e77d9dda9166c90a981b827899837bdf3ceef438f3b3a73a0586fc26).
  The battery stands at 25 of 25 cells
  passing (battery27_results.txt), with
  C17 to C22 at VL-100's dispositions:
  the unavailable cells closing with
  their contracted reasons, the judge's
  answer naming an unpinned model, its
  choice answer malformed, its evidence
  answer malformed, its risk answer
  malformed at C20 and again at C22, and
  the bounds cell scoring 10 and 0; the
  fake judge stands byte-identical at
  its contracted md5
  a79ded49ae7285fd2d7b028c8b47dd7f. The
  walk stands at 10 of 10 cells passing
  (walk27_results.txt; its bank's
  summary line keeps the instrument's
  standing WALK13 label). The pin stands
  re-verified: flighttape commit
  74a83298, tree afd4d7d1, 50 blobs
  confirmed against the live tree at
  this round's census, its suite at 241
  tests passing across 19 suites in the
  chain's pin bank (pin_suite_l2.log and
  pin_summary_l2.txt), and the golden
  digest cc9c4e01 standing in the pinned
  source, read at this lane's hand in
  tests/digest_cli.rs. The oracle stands
  re-verified: commit d62e5dc, tree
  74394802, 24 blobs, the census
  recomputing its blobs against the
  saved tree census with none bad. Each
  measurement bank stands verified whole
  at this hand (exact PASS counts, no
  duplicated cell, byte-identical to its
  stdout bank).
- VL-179. The local gate at base and
  head trees, in the VL-119 form, with
  whole-tree scan coverage standing and
  the retention form discharged. At the
  base, gate_base_launch2.log in the
  chain's banks: cargo test 47 tests
  passing in the 3 library, 24 judgment,
  20 registry split, clippy and fmt at
  exit 0. At the head,
  gate_head_launch4.log: VERIFY exit 0
  (21 files, bad none), cargo test 47 in
  the same 3 plus 24 plus 20 split,
  TEST, CLIPPY, FMT, and SCAN exits all
  0. The codepoint scan at both trees
  takes an explicit per-file list
  expanded from the tree listing and
  covers every text file in the tree, 21
  of 21 files at both trees, the gate
  log printing the scanned count against
  the tree's file count; the six files
  Round 24's scan omitted
  (.github/workflows/ci.yml, .gitignore,
  Cargo.lock, LICENSE,
  scripts/codepoint_scan.py,
  tests/fixtures/judgments_synthetic.json)
  stand in the list by name, and no file
  is excluded. The scan's planted
  control convicted inside each gate log
  (control 1 and 1), totals 0 and 0 at
  both trees. The retention form stands
  discharged: the gate scripts bank each
  launch's output under its own
  launch-numbered file
  (gate_base_launch2.log,
  gate_head_launch4.log), no launch's
  log overwrites another's, and a launch
  that reaches its gate banks its output
  under its own name even when it counts
  nowhere; a launch that dies before its
  gate banks no log, and what stands for
  it is its status evidence, the head
  gate's launch 1 at its slot line in
  gate_head27_status.txt and the base
  chain's launch 1 in this item and the
  lane record, its status file never
  created. Holds and launches: the
  round's build-slot holds stand counted
  in the lane record (the base chain's
  hold at its launch 2 and the head
  gate's holds at launches 1, 2, and 4,
  its launch 3 dead queued without
  acquiring). The base chain's launch 1
  died queued on the build slot in a
  runtime restart (SIGTERM at about
  twenty-six minutes, never acquiring,
  no stage run, no bank, its status file
  never created), counted dead and
  relaunched in the same sitting; launch
  2 acquired at 19:31:48Z and ran the
  chain whole (verify, build, base gate,
  census, and the pin suite, every stage
  at exit 0). The head gate's launch 1
  queued on the build slot behind the
  fleet's holds, acquired at 21:10:03Z,
  probed the shut write wave without
  opening, and died in a runtime restart
  at about 21:59Z with no gate run and
  no log banked, its slot line standing
  in gate_head27_status.txt, counted
  dead; launch 2 ran against the head
  checkout rebuilt from the repaired
  closure tree 3ac9a059 and verified
  file by file, and passed whole,
  standing until this item's wording was
  tightened at step 0; launch 3 queued
  on the build slot behind the fleet's
  holds and died in a runtime restart
  without acquiring, no gate run and no
  status line, counted dead; launch 4
  ran against the final head checkout,
  rebuilt from the tree carrying this
  accounting and verified file by file,
  and passed whole, its log the standing
  one. No other gate launch died or
  failed.
- VL-180. Fresh surface work, proposed
  in the lane record before it ran. The
  text forms in the free-text fields
  (source, proposal, and baseline at the
  open; rationale at the close), a twin
  surface Rounds 14 through 26 did not
  measure, twenty cells on genuinely
  independent arms
  (textforms27_results.txt), standing at
  20 of 20 agree, 0 failed, 0 blocked.
  Every earlier surface celled measures,
  timestamps, registry states, margins,
  and refusals, and the battery's texts
  stand fixed and plain; this one cells
  the renderings of the texts
  themselves, the derivation read from
  the shipped source before running (the
  oracle's json.dumps with
  ensure_ascii=False for the event lines
  and the views, the port's emit.rs
  mirroring that encoder exactly, both
  CLIs passing the text arguments
  verbatim, the registry validating the
  four fields only as non-empty
  strings). Accepted forms stand
  verbatim in each arm's own event at
  its position, in the encoder's derived
  form: a double quote and a backslash
  escaped in the source, a single quote
  and a solidus standing raw in the
  proposal, a tab and a newline escaped
  in the baseline, accented Latin and
  CJK standing raw as UTF-8 in the
  proposal, two controls without short
  escapes (U+0001 and U+000B) rendered
  as four-digit hexadecimal escapes in
  the source, a carriage return escaped
  and DEL (U+007F) standing raw in the
  baseline, an astral emoji and a
  combining accent standing raw in the
  first close's rationale, and the
  backspace and form feed short escapes
  in the second close's rationale. One
  refusal bounds the surface: an open
  with an empty proposal refused at line
  5 on both arms at exit 1 with stderr
  byte-equal to the contracted line
  (error, line 5: proposal is not a
  non-empty string), stdout empty, the
  registry byte-unchanged at four lines.
  The closes rest at their contracted
  constants on each arm's own registry:
  (1, 5) keep, (2, 6) keep, (3, 7) rerun
  with rescue regressed, (4, 8) keep,
  and (9, 10) keep; the two shows read
  the celled texts back byte-identical
  through each arm's own parser with the
  derived escape forms standing in the
  view bytes; the list shows the five
  closed and check exits 0 with the
  registry at fifteen lines.
- VL-181. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-182. Closure. README moves to 27 of
  100 in the closure commit only; slice
  and closure staged in separate
  directories (stage_slice,
  stage_closure); CONTRACT append-only
  by byte-prefix; the tree diff base to
  head touches exactly CONTRACT.md and
  README.md. The round's weather, by
  name: the write wave stood shut at the
  seating for exec writes of any real
  size, routing the seating reads
  through the file tool and the stdout;
  the first extraction sequence ran
  about twenty-five minutes against the
  fluttering wave, the base extraction's
  first three files failing their writes
  in the opening shut stretch and the
  first read-back launch failing at its
  first bank, both discharged by a
  relaunch that landed every file (the
  base extraction at 21 of 21 blobs, the
  oracle at 24 of 24, the read-back
  complete); the base chain's launch 1
  died queued on the build slot in a
  runtime restart, counted dead and
  relaunched in the same sitting, its
  launch 2 running the chain whole; the
  measurements ran once and passed whole
  at their first launch; the head gate's
  launch 1 died in a runtime restart
  after acquiring the slot and probing
  the shut wave (no gate run, counted
  dead), its launch 2 passed whole
  against the repaired closure, its
  launch 3 died queued on the build slot
  in a runtime restart without acquiring
  (no gate run, counted dead), and its
  launch 4 passed whole against the
  final tree after the repairs the
  restarts and step 0 required (the
  retention claim qualified, the head
  gate's covered tree named). The API
  pool drained fleet-wide through the
  extraction sequence (3,278 remaining
  at the seating probe, 569 at the
  read-back relaunch's probe); no
  rate-limit stop hit this lane. No
  launch's failure is counted as
  evidence anywhere in this round's
  items.
- VL-183. The read-back of Round 27's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 46 stands closed and
  merged: its body stands at 16,338
  characters at this hand, the lane's
  body of 12,989 characters with the
  owner completion record appended after
  it, its head
  37e6222f95820f65af8b78ce7eef820dfb136931
  over repair
  9cad708c4f86306985b0e75ba31b1ca812e9323e
  over repair
  c0e45062d08550cfbd22c957b14eac09e5c97142
  over closure
  09d2879639b9044f71932b30f2a8e91d64a9c5de
  over slice
  fb760623c0baeead2dde821d8b4e423280fcfc6a
  over the base
  6182d23e9086a817de89b2fde307dc320b9f71a3,
  a five-commit chain, and it landed
  through the merge queue as squash
  commit
  c2c7df2dba571ba5855e10ec9910f1ba0845ce91,
  single parent 6182d23e, the landing
  tree
  e0ae917ad95ba624587517342d556a92ce64118a
  verified by the owner as the reviewed
  head tree itself and re-read at this
  round's hand (main stands at c2c7df2d
  on a fresh read, its tree at e0ae917a
  over twenty-one blobs, not truncated),
  and the README read on main at 27 of
  100. The census at this read stands at
  four issue comments, one review, and
  two review comments: the trunk note
  7115389892 (113 characters, created
  2026-11-23T22:09:06Z, last edited
  2026-11-23T23:50:08Z, its text now the
  merged form), the CodeRabbit shell
  7115394078 cited in drift-aware form
  (11,093 characters at this hand,
  created 2026-11-23T22:09:18Z, last
  edited 2026-11-23T23:42:05Z; the Round
  27 lane read 11,091 characters in its
  re-review form, edited
  2026-11-23T23:35:38Z, and the owner
  read 11,093 characters at the later
  edit, the two characters of re-render
  drift standing between the lane's read
  and the later reads; Run ID
  cfbe046b-0b08-443d-8e19-a0366057821e,
  profile CHILL, Merge Risk Minimal in
  the re-review form, coverage at the
  exact head 37e6222f, four pre-merge
  checks passed, no actionable comments
  in the re-review form; its first form
  stands at 8,649 characters at the
  lane's hand, edited
  2026-11-23T22:12:09Z, Merge Risk Low,
  one review with two inline Minors,
  both on VL-179; the shell read whole
  at this round's hand and standing
  banked whole in this round's scratch
  from the read-back), the merge comment
  7117262931 (12 characters, created
  2026-11-23T23:42:10Z), and the landing
  comment 7117430234 (3,305 characters,
  created 2026-11-23T23:51:00Z) carrying
  this round's charges; nothing stands
  after the landing comment. The one
  review stands at 6030345729,
  COMMENTED, submitted
  2026-11-23T22:12:05Z, with two review
  comments on CONTRACT.md (4672717625
  and 4672717636), the shell's two
  inline Minors. The owner's completion
  record stands read with it: spec
  fidelity APPROVE with two Minors,
  standards APPROVE with one Minor, and
  red team APPROVE with no findings at
  any grade and no carried limitation,
  every charged leg run at its hand, its
  composed twenty-first text-forms case
  agreeing on all three forms at its
  hand, no Blocker and no Major at any
  hand. Its account stands contracted
  here. The chain's three repair commits
  each carry their named accounting in
  the CONTRACT itself: c0e45062 accounts
  the head gate's launch 1, dead in a
  runtime restart after acquiring the
  slot at 21:10:03Z and probing the shut
  write wave; 9cad708c carries the step
  0 tightening of VL-179 (one finding
  fixed as valid, one dismissed on its
  premise with its wording point fixed);
  37e6222f accounts the head gate's
  launch 3, dead queued on the build
  slot about seventy-five minutes in a
  second runtime restart without
  acquiring. The retention form stands
  in its qualified final form: every
  launch that reached its gate banks its
  own log under its own launch-numbered
  name with no overwriting anywhere
  (gate_base_launch2.log at the base
  tree, gate_head_launch2.log against
  the repaired closure tree 3ac9a059,
  gate_head_launch4.log against the
  final tree e0ae917a, each read whole
  at three hands at 47 tests in the
  split with the whole-tree scan and the
  convicted control), and the three
  launches that died before their gates
  stand on their named status evidence
  (the base chain's launch 1 in the item
  and the lane record, its status file
  never created; the head gate's launch
  1 at its slot line in
  gate_head27_status.txt; the head
  gate's launch 3 on the status bank's
  silence, the script writing its slot
  line only after the lock returns). The
  text-forms surface stands at 20 of 20
  agree on its first run, its refusal
  bounding the surface at line 5, and
  the three Round 26 corrections stand
  as carried in VL-177 (the rejection
  entry's scan-list limb, VL-173's
  closing census sentence, VL-173's
  same-closure-tree phrase). The round's
  three corrections stand carried here,
  each in one line. The body's retention
  wording stands corrected: the claim
  stands in its qualified form, and the
  unqualified sentence in the body's
  VL-179 paragraph does not stand
  against the item's final text. The
  body's prefix summary stands
  corrected: the prior rounds' base text
  stands intact as a prefix at every
  step, and Round 27's repairs amended
  its own additions in place. VL-179's
  lead and naming stand corrected: the
  retention form stands discharged in
  its qualified form, and the standing
  banks are named, gate_head_launch2.log
  for head launch 2 and the status
  bank's silence for head launch 3.
- VL-184. Standing regression, in the
  VL-119 form. Every measurement ran
  against the chain binary built fresh
  in this round from the blob-verified
  base checkout (sha256
  45bb3bba0f199c43f8914d081cbf344959378247ddb73a571dafbe112f2e3550).
  The battery stands at 25 of 25 cells
  passing (battery28_results.txt), with
  C17 to C22 at VL-100's dispositions:
  the unavailable cells closing with
  their contracted reasons, the judge's
  answer naming an unpinned model, its
  choice answer malformed, its evidence
  answer malformed, its risk answer
  malformed at C20 and again at C22, and
  the bounds cell scoring 10 and 0; the
  fake judge stands byte-identical at
  its contracted md5
  a79ded49ae7285fd2d7b028c8b47dd7f. The
  walk stands at 10 of 10 cells passing
  (walk28_results.txt; its bank's
  summary line keeps the instrument's
  standing WALK13 label). The pin stands
  re-verified: flighttape commit
  74a83298, tree afd4d7d1, 50 blobs
  confirmed against the live tree at
  this round's census, its suite at 241
  tests passing across 19 suites in the
  chain's pin bank (pin_suite_l2.log and
  pin_summary_l2.txt), and the golden
  digest cc9c4e01 standing in the pinned
  source, read at this lane's hand in
  tests/digest_cli.rs. The oracle stands
  re-verified: commit d62e5dc, tree
  74394802, 24 blobs, the census
  recomputing its blobs against the
  saved tree census with none bad. Each
  measurement bank stands verified whole
  at this hand (exact PASS counts, no
  duplicated cell, byte-identical to its
  stdout bank).
- VL-185. The local gate at base and
  head trees, in the VL-119 form, with
  whole-tree scan coverage standing and
  the retention form standing in its
  qualified final form. At the base,
  gate_base_launch2.log in the chain's
  banks: cargo test 47 tests passing in
  the 3 library, 24 judgment, 20
  registry split, clippy and fmt at exit
  0. At the head, gate_head_launch1.log:
  VERIFY exit 0 (21 files, bad none),
  cargo test 47 in the same 3 plus 24
  plus 20 split, TEST, CLIPPY, FMT, and
  SCAN exits all 0. The codepoint scan
  at both trees takes an explicit
  per-file list expanded from the tree
  listing and covers every text file in
  the tree, 21 of 21 files at both
  trees, the gate log printing the
  scanned count against the tree's file
  count; the six files Round 24's scan
  omitted (.github/workflows/ci.yml,
  .gitignore, Cargo.lock, LICENSE,
  scripts/codepoint_scan.py,
  tests/fixtures/judgments_synthetic.json)
  stand in the list by name, and no file
  is excluded. The scan's planted
  control convicted inside each gate log
  (control 1 and 1), totals 0 and 0 at
  both trees. The retention form stands
  in its qualified final form: the gate
  scripts bank each launch's output
  under its own launch-numbered file
  (gate_base_launch2.log,
  gate_head_launch1.log), no launch's
  log overwrites another's, and a launch
  that reaches its gate banks its output
  under its own name even when it counts
  nowhere; a launch that dies before its
  gate banks no log, and what stands for
  it is its status evidence, the base
  chain's launch 1 in this item and the
  lane record, its status file never
  created. Holds and launches: the
  round's build-slot holds stand counted
  in the lane record (the base chain's
  hold at its launch 2 and the head
  gate's hold at its launch 1). The base
  chain's launch 1 died queued on the
  build slot in a runtime restart
  (SIGTERM at about eight and a half
  minutes, never acquiring, no stage
  run, no bank, its status file never
  created), counted dead and relaunched
  in the same sitting; launch 2 queued
  behind the fleet's holds for about
  fifty-seven minutes, acquired at
  01:06:14Z, and ran the chain whole
  (verify, build, base gate, census, and
  the pin suite, every stage at exit 0).
  The head gate's launch 1 ran against
  the head checkout rebuilt from the
  pushed closure tree and verified file
  by file, and passed whole, its log the
  standing one. No other gate launch
  died or failed.
- VL-186. Fresh surface work, proposed
  in the lane record before it ran. The
  experiment id forms, a twin surface
  Rounds 14 through 27 did not measure,
  twenty-four cells on genuinely
  independent arms
  (idforms28_results.txt), standing at
  24 of 24 agree, 0 failed, 0 blocked.
  Every earlier surface celled measures,
  timestamps, registry states, margins,
  refusals on standing ids, and the
  free-text fields' renderings, and
  every experiment stood under a plain
  hyphenated id; this one cells the id
  itself, the derivation read from the
  shipped source before running
  (event_defects in both arms validating
  the experiment only as a non-empty
  string, the fold in both arms keying
  states by the exact string, both CLIs
  passing the experiment positional
  verbatim, the encoder rendering the
  experiment field as VL-180 celled it
  at the other fields). The accepted
  forms stand verbatim in each arm's own
  event at its position: a space in the
  first id, a slash with dots in the
  second, a numeric-looking id standing
  as a string, a unicode id (CJK with
  accented Latin) standing raw as UTF-8,
  a double quote and a backslash escaped
  in the fifth, a case variant of the
  first standing as a distinct
  experiment, and a 180-character id
  standing whole. Two refusals bound the
  surface: an open with an empty id
  refused at line 8 on both arms at exit
  1 with stderr byte-equal to the
  contracted line (error, line 8:
  experiment is not a non-empty string),
  and a show on an absent space-carrying
  id refused at exit 1 with stderr
  byte-equal to its contracted line
  (error, no experiment exp 28 omega),
  stdout empty on both arms and the
  registry byte-unchanged by each. The
  closes rest at their contracted
  constants on each arm's own registry:
  (1, 8) keep, (3, 9) keep, (4, 10)
  keep, (5, 11) rerun with rescue
  regressed, and (6, 12) keep; the slash
  and long ids stand open; the two case
  shows return their own opened
  positions, 1 and 6, proving the fold
  case-sensitive at the read path; the
  list shows the seven in open order and
  check exits 0 with the registry at
  seventeen lines.
- VL-187. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-188. Closure. README moves to 28 of
  100 in the closure commit only; slice
  and closure staged in separate
  directories (stage_slice,
  stage_closure); CONTRACT append-only
  in the standing sense, the prior
  rounds' text standing intact as a
  prefix and this round's own additions
  amended in place only where a repair's
  named accounting requires it; the tree
  diff base to head touches exactly
  CONTRACT.md and README.md. The round's
  weather, by name: the base chain's
  launch 1 died queued on the build slot
  in a runtime restart, counted dead and
  relaunched in the same sitting, its
  launch 2 queuing behind the fleet's
  holds for about fifty-seven minutes
  before running the chain whole; the
  measurements ran once and passed whole
  at their first launch; the extractions
  landed whole on their first passes,
  the write wave recording no failed
  write against this round's seating.
  The API pool drained fleet-wide
  through the round's reads (2,094
  remaining at the seating probe, 1,797
  at the oracle extraction's probe); no
  rate-limit stop hit this lane. No
  launch's failure is counted as
  evidence anywhere in this round's
  items.
- VL-189. The read-back of Round 28's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 48 stands closed and
  merged: its body stands at 12,784
  characters at this hand, the lane's
  body of 9,692 characters with the
  owner completion record appended after
  it, its head
  f6751877a8863d734bfc18932a3992ffe5d1ab71
  over slice
  d03d105ba1fe10341e10d0dd66c3d781f9f3ab06
  over the base
  c2c7df2dba571ba5855e10ec9910f1ba0845ce91,
  a clean two-commit chain with no
  repair commits, and it landed through
  the merge queue as squash commit
  b1b8ca575abbc5064a9ee8790b03725381257031,
  single parent c2c7df2d, the landing
  tree
  dc9874aecaf5940927cf95b8753ac6260d31f811
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  b1b8ca57 on a fresh read, its tree at
  dc9874ae over twenty-one blobs, not
  truncated), and the README read on
  main at 28 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7119456047 (113 characters,
  created 2026-11-24T01:47:53Z, last
  edited 2026-11-24T02:03:14Z, its text
  now the merged form), the CodeRabbit
  shell 7119459479 cited in drift-aware
  form (10,960 characters at this hand,
  created 2026-11-24T01:48:06Z, last
  edited 2026-11-24T01:52:19Z; the Round
  28 lane read 10,958 characters in its
  finished form, edited
  2026-11-24T01:50:00Z, and the owner
  read 10,960 characters at the later
  edit, the two characters of re-render
  drift standing between the lane's read
  and the later reads; Run ID
  22e21a8a-ce73-459f-b9cf-02ede6d67130,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head f6751877,
  four pre-merge checks passed, no
  actionable comments, no review object,
  and no inline comments, its title
  check passed; the shell read whole at
  this round's hand and standing banked
  whole in this round's scratch from the
  read-back), the merge comment
  7119576024 (12 characters, created
  2026-11-24T01:54:58Z), and the landing
  comment 7119745559 (2,997 characters,
  created 2026-11-24T02:05:32Z) carrying
  this round's charges; nothing stands
  after the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with one Minor,
  standards APPROVE with two Minors, and
  red team APPROVE with no findings at
  any grade and no carried limitation,
  the second consecutive clean red team
  verdict on this repo, every charged
  leg run at its hand including a
  composed twenty-fifth id-forms case, a
  trailing-dot id, agreeing on every
  step at its hand, no Blocker and no
  Major at any hand. Its account stands
  contracted here. The chain needed no
  repair commits: the staged items stood
  exactly as written through staging,
  both gates, CI, and step 0. The
  retention form stands in its qualified
  final form: every launch that reached
  its gate banks its own log under its
  own launch-numbered name with no
  overwriting anywhere
  (gate_base_launch2.log at the base
  tree, gate_head_launch1.log at the
  head tree, the head gate passing whole
  at its first launch against the
  checkout rebuilt from the pushed
  closure tree), and the one launch that
  died before its gate stands on its
  named status evidence (the base
  chain's launch 1, dead queued on the
  build slot in a runtime restart, its
  status file never created, counted
  dead and relaunched in the same
  sitting; its launch 2 queued behind
  the fleet's holds for about
  fifty-seven minutes and ran the chain
  whole; the head gate's launch 1 queued
  for about thirty-one minutes, acquired
  at 01:46:03Z, and passed whole). The
  id-forms surface stands at 24 of 24
  agree on its first run, both refusals
  byte-equal to their contracted lines,
  and the three Round 26 corrections
  stand as carried in VL-183. Round 28's
  event stands carried here in one line
  in its exact form: the opened body
  read back 612 chars longer than
  submitted (9,520 against 8,908), an
  external append whose content and
  origin stand unestablished (no banked
  pre-patch body, no body-version
  history, no timeline body edit),
  repaired by the lane's patch restoring
  the intended text verified identical
  at 9,692 chars, with no figure resting
  on the appended material. The Minors
  converge on that event's placement
  (the spec Minor and the first
  standards Minor: it stood in neither
  durable text, VL-188's weather list
  omitting it and the body carrying no
  mention), and the remaining standards
  Minor is a doubled label in the body's
  step 0 paragraph ("Step 0: Step 0:"),
  misstating nothing.
- VL-190. Standing regression, in the
  VL-119 form. Every measurement ran
  against the chain binary built fresh
  in this round from the blob-verified
  base checkout (sha256
  30d719c0ea133aba2702cd177e7a80fcbe512858a6105fe2ddfd44df767d0a3e).
  The battery stands at 25 of 25 cells
  passing (battery29_results_l2.txt),
  with C17 to C22 at VL-100's
  dispositions: the unavailable cells
  closing with their contracted reasons,
  the judge's answer naming an unpinned
  model, its choice answer malformed,
  its evidence answer malformed, its
  risk answer malformed at C20 and again
  at C22, and the bounds cell scoring 10
  and 0; the fake judge stands
  byte-identical at its contracted md5
  a79ded49ae7285fd2d7b028c8b47dd7f. The
  walk stands at 10 of 10 cells passing
  (walk29_results_l2.txt; its bank's
  summary line keeps the instrument's
  standing WALK13 label). The pin stands
  re-verified: flighttape commit
  74a83298, tree afd4d7d1, 50 blobs
  confirmed against the live tree at
  this round's census, its suite at 241
  tests passing across 19 suites in the
  chain's pin bank (pin_suite_l2.log and
  pin_summary_l2.txt), and the golden
  digest cc9c4e01 standing in the pinned
  source, read at this lane's hand in
  tests/digest_cli.rs. The oracle stands
  re-verified: commit d62e5dc, tree
  74394802, 24 blobs, the census
  recomputing its blobs against the
  saved tree census with none bad. Each
  measurement bank stands verified whole
  at this hand (exact PASS counts, no
  duplicated cell, byte-identical to its
  launch's stdout bank).
- VL-191. The local gate at base and
  head trees, in the VL-119 form, with
  whole-tree scan coverage standing and
  the retention form standing in its
  qualified final form. At the base,
  gate_base_launch2.log in the chain's
  banks: cargo test 47 tests passing in
  the 3 library, 24 judgment, 20
  registry split, clippy and fmt at exit
  0. At the head, gate_head_launch1.log:
  VERIFY exit 0 (21 files, bad none),
  cargo test 47 in the same 3 plus 24
  plus 20 split, TEST, CLIPPY, FMT, and
  SCAN exits all 0. The codepoint scan
  at both trees takes an explicit
  per-file list expanded from the tree
  listing and covers every text file in
  the tree, 21 of 21 files at both
  trees, the gate log printing the
  scanned count against the tree's file
  count; the six files Round 24's scan
  omitted (.github/workflows/ci.yml,
  .gitignore, Cargo.lock, LICENSE,
  scripts/codepoint_scan.py,
  tests/fixtures/judgments_synthetic.json)
  stand in the list by name, and no file
  is excluded. The scan's planted
  control convicted inside each gate log
  (control 1 and 1), totals 0 and 0 at
  both trees. The retention form stands
  in its qualified final form: the gate
  scripts bank each launch's output
  under its own launch-numbered file
  (gate_base_launch2.log,
  gate_head_launch1.log), no launch's
  log overwrites another's, and a launch
  that reaches its gate banks its output
  under its own name even when it counts
  nowhere; a launch that dies before its
  gate banks no log, and what stands for
  it is its status evidence, the base
  chain's launch 1 on
  chain29_launch1_evidence.txt and the
  lane record, its status append failed
  under the write wave and
  chain29_status.txt standing zero-byte
  at the lane's next read. Holds and
  launches: the base chain's launch 1
  queued on the build slot from 02:14Z
  behind the fleet's holds, acquired at
  about 03:23Z after about sixty-nine
  minutes, and died at its first status
  write under the box's write wave (the
  slot-acquired line, echo I/O error),
  no stage run and no bank created,
  counted dead and relaunched in the
  same sitting; launch 2 queued for
  about six minutes, acquired at
  03:37:23Z, and ran the chain whole
  (verify, build, base gate, census, and
  the pin suite, every stage at exit 0).
  The head gate's launch 1 ran against
  the head checkout rebuilt from the
  pushed closure tree and verified file
  by file, and passed whole, its log the
  standing one. No other gate launch
  died or failed.
- VL-192. Fresh surface work, proposed
  in the lane record before it ran. The
  judged list forms at the open, a twin
  surface Rounds 14 through 28 did not
  measure, twenty-eight cells on
  genuinely independent arms
  (judgedforms29_results_l2.txt),
  standing at 28 of 28 agree, 0 failed,
  0 blocked. Every earlier surface
  celled measures, timestamps, registry
  states, margins as event arguments,
  refusals on standing judged lists, and
  the id and free-text forms, and the
  judged list itself stood plain
  throughout (unique string names from
  the baseline measures, directions
  valid, entry margins valid or absent);
  this one cells the judged list's own
  forms, the derivation read from the
  shipped source before running
  (event_defects in both arms validating
  the judged list in a fixed order ahead
  of the per-name membership checks, the
  suggestion rule in both arms iterating
  the judged list in its own order with
  the entry margin ahead of the event
  margin ahead of the default of 10).
  Ten refused opens bound the surface on
  the empty registry, each at exit 1 on
  both arms with stderr byte-equal to
  its contracted line and the registry
  absent on both arms after each:
  duplicate judged names; a judged name
  absent from the baseline measures; a
  direction outside lower and higher; an
  entry that is not an object; an entry
  name that is not a string; an entry
  margin that is negative; an entry
  margin that is a string; an empty
  judged list; a judged value that is
  not a list; and an empty baseline
  measures object, the one cell carrying
  two defects, its stderr the joined
  line naming the empty object and the
  absent judged name. The accepted
  writes stand on both arms' own
  registries: a subset open judging cost
  alone over a three-measure baseline
  closes keep at (1, 2) with no measure
  regressed and none unmeasured, the
  unjudged measures taking no part in
  the verdict though one improved and
  one worsened past any margin; an order
  open judging rescue ahead of cost
  closes kill at (4, 5) with the
  regressed list standing in judged
  order, rescue ahead of cost; a
  precedence open under an event margin
  of 50 closes rerun at (7, 8) with cost
  regressed on its entry margin of 5
  while rescue's rise of 20 percent
  stands inside the event margin it
  falls back to; a zero-margin entry
  closes rerun at (10, 11), cost
  regressed on a rise of 1 against its
  margin of 0 while rescue's fall stands
  inside the default; an unmeasured open
  closes keep at (13, 14) with rescue
  standing unmeasured in judged order
  and the suggestion resting on the
  measured names alone. The show of the
  subset experiment returns its own
  events on both arms, the list shows
  the five experiments in open order
  with their verdicts and suggestions
  agreeing (keep, kill, rerun, rerun,
  keep), check exits 0 on both arms, and
  the registry stands at fifteen lines
  on both arms. The surface's own first
  launch stood at 27 of 28: its J10 cell
  as first written passed the two-entry
  judged list where the proposal
  contracts the cost entry alone, so
  both arms answered with the
  three-defect joined line and the cell
  failed on its contracted line alone,
  the arms agreeing with each other
  throughout; the cell was conformed to
  the proposal, the first launch's banks
  stand preserved under their own names,
  and the surface stands at its second
  launch's figures above.
- VL-193. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-194. Closure. README moves to 29 of
  100 in the closure commit only; slice
  and closure staged in separate
  directories (stage_slice,
  stage_closure); CONTRACT append-only
  in the standing sense, the prior
  rounds' text standing intact as a
  prefix and this round's own additions
  amended in place only where a repair's
  named accounting requires it; the tree
  diff base to head touches exactly
  CONTRACT.md and README.md. The round's
  weather, by name: the base chain's
  launch 1 queued on the build slot for
  about sixty-nine minutes behind the
  fleet's holds, acquired, and died at
  its first status write under the box's
  write wave, counted dead and
  relaunched in the same sitting, its
  launch 2 queuing about six minutes and
  running the chain whole; the write
  wave that killed it was survived by
  clearing the download caches and
  probing inside the sitting, the probe
  failing through 03:29Z and a runtime
  restart interrupting it before writes
  verified healthy again at 03:30Z; the
  measurements' first launch stood at 27
  of 28 on the judged surface's J10 cell
  as first written, a driver slip
  against the recorded proposal with the
  two implementations agreeing
  throughout, the cell conformed and the
  measurements relaunched whole; the
  extractions landed whole on their
  first passes. The API pool stood tight
  through the round's reads (610
  remaining at the seating probe, 550
  after the read-back batch) and drained
  fleet-wide; no rate-limit stop hit
  this lane. No launch's failure is
  counted as evidence anywhere in this
  round's items.
- VL-195. The read-back of Round 29's
  landing pull request stands contracted
  here on its closed record, read whole
  through the API at this round's hand.
  Pull request 50 stands closed and
  merged: its body stands at 14,216
  characters at this hand, the lane's
  body of 11,205 characters with the
  owner completion record appended after
  it, its head
  54e6568c4140c95bf428b199abe84760a4257a4f
  over slice
  0845afcfae9d10a160622ff7f40ee8901deb6f81
  over the base
  b1b8ca575abbc5064a9ee8790b03725381257031,
  a clean two-commit chain with no
  repair commits, and it landed through
  the merge queue as squash commit
  4f5a9be2a4d5419296879d31e47321f53f7428ce,
  single parent b1b8ca57, the landing
  tree
  85c14e89cb1842688efb7f170df79cc47d1bc628
  verified by the owner as the reviewed
  closure head tree itself and re-read
  at this round's hand (main stands at
  4f5a9be2 on a fresh read, its tree at
  85c14e89 over twenty-one blobs, not
  truncated), and the README read on
  main at 29 of 100. The census at this
  read stands at four issue comments, no
  reviews, and no review comments: the
  trunk note 7123008189 (113 characters,
  created 2026-11-24T05:30:12Z, last
  edited 2026-11-24T05:40:00Z, its text
  now the merged form), the CodeRabbit
  shell 7123011371 cited in drift-aware
  form (11,321 characters at this hand,
  created 2026-11-24T05:30:25Z, last
  edited 2026-11-24T05:32:53Z; the Round
  29 lane read 11,319 characters in its
  finished form, edited
  2026-11-24T05:31:59Z, and the owner
  read 11,321 characters at the later
  edit, the two characters of re-render
  drift standing between the lane's read
  and the later reads; Run ID
  f55372ea-dcb3-4074-a544-47cf22d74d2e,
  profile CHILL, Merge Risk Minimal,
  coverage at the exact head 54e6568c,
  four pre-merge checks passed, no
  actionable comments, no review object,
  and no inline comments, its title
  check passed; the shell read whole at
  this round's hand and standing banked
  whole in this round's scratch from the
  read-back), the merge comment
  7123050598 (12 characters, created
  2026-11-24T05:32:54Z), and the landing
  comment 7123197306 (3,138 characters,
  created 2026-11-24T05:42:24Z) carrying
  this round's charges; nothing stands
  after the landing comment. The owner's
  completion record stands read with it:
  spec fidelity APPROVE with no
  findings, standards APPROVE with no
  findings, and red team APPROVE with no
  findings at any grade and no carried
  limitation, the cleanest verdict set
  the bench has recorded on this repo
  and the third consecutive clean red
  team verdict, every charged leg run at
  its hand including the re-run of the
  J10 cell in both its forms, each
  reproduced byte-equal, and a composed
  twenty-ninth case, a baseline measure
  whose value is an object, refused
  identically on both arms; no Blocker,
  no Major, and no Minor at any hand.
  Its account stands contracted here.
  The chain needed no repair commits.
  The retention form stands in its
  qualified final form: every launch
  that reached its gate banks its own
  log under its own launch-numbered name
  with no overwriting anywhere
  (gate_base_launch2.log at the base
  tree, gate_head_launch1.log at the
  head tree, the head gate passing whole
  at its first launch against the
  checkout rebuilt from the pushed
  closure tree after a queue of about
  one hundred and one minutes), and the
  one launch that died before its gate
  stands on its named evidence note
  (chain29_launch1_evidence.txt: the
  base chain's launch 1 queued about
  sixty-nine minutes, acquired, and died
  at its first status write under the
  write wave with the wrapper's echo I/O
  error, its status file left zero-byte,
  the note quoting the wrapper's stderr
  verbatim, the quote matched by the red
  team against line 18 of chain29.sh
  itself, no figure resting on it). The
  judged list forms surface stands at 28
  of 28 on the conformed second launch,
  its amendment standing as the bench's
  model of the form: the proposal
  recorded before running contracts the
  cost entry alone at J10, launch 1's
  driver as first written passed a
  two-entry judged list and both
  implementations answered with the
  three-defect joined line, agreeing
  with each other against the proposal,
  the cell conformed to the proposal,
  launch 1's banks preserved under their
  own names at 27 of 28, and launch 2
  the conformed surface's first clean
  run. Round 28's external-append event
  stands carried in VL-189 in one line
  in its exact form, and the opened body
  of Round 29 read back byte-identical
  with no external append, its step 0
  label standing single in the patched
  body, Round 28's doubled-label Minor
  discharged in fact.
- VL-196. Standing regression, in the
  VL-119 form. Every measurement ran
  against the chain binary built fresh
  in this round from the blob-verified
  base checkout (sha256
  0bf0c78b0ed5357956f13cbb11b8b6aa64371524889ab8f19d59fe3549ea67fd).
  The battery stands at 25 of 25 cells
  passing (battery30_results_l1.txt; its
  bank's summary line keeps the
  instrument's standing BATTERY28 label,
  the figures this round's own), with
  C17 to C22 at VL-100's dispositions:
  the unavailable cells closing with
  their contracted reasons, the judge's
  answer naming an unpinned model, its
  choice answer malformed, its evidence
  answer malformed, its risk answer
  malformed at C20 and again at C22, and
  the bounds cell scoring 10 and 0; the
  fake judge stands byte-identical at
  its contracted md5
  a79ded49ae7285fd2d7b028c8b47dd7f. The
  walk stands at 10 of 10 cells passing
  (walk30_results_l1.txt; its bank's
  summary line keeps the instrument's
  standing WALK13 label). The pin stands
  re-verified: flighttape commit
  74a83298, tree afd4d7d1, 50 blobs
  confirmed against the live tree at
  this round's census, its suite at 241
  tests passing across 19 suites in the
  chain's pin bank (pin_suite_l2.log and
  pin_summary_l2.txt), and the golden
  digest cc9c4e01 standing in the pinned
  source, read at this lane's hand in
  tests/digest_cli.rs. The oracle stands
  re-verified: commit d62e5dc, tree
  74394802, 24 blobs, the census
  recomputing its blobs against the
  saved tree census with none bad. Each
  measurement bank stands verified whole
  at this hand (exact PASS counts, no
  duplicated cell, byte-identical to its
  launch's stdout bank).
- VL-197. The local gate at base and
  head trees, in the VL-119 form, with
  whole-tree scan coverage standing and
  the retention form standing in its
  qualified final form. At the base,
  gate_base_launch2.log in the chain's
  banks: verify exit 0 (21 blobs, bad
  none), cargo test 47 tests passing in
  the 3 library, 24 judgment, 20
  registry split, clippy and fmt at exit
  0. At the head, gate_head_launch2.log:
  VERIFY exit 0 (21 files, bad none),
  cargo test 47 in the same 3 plus 24
  plus 20 split, TEST, CLIPPY, FMT, and
  SCAN exits all 0. The codepoint scan
  at both trees takes an explicit
  per-file list expanded from the tree
  listing and covers every text file in
  the tree, 21 of 21 files at both
  trees, the gate log printing the
  scanned count against the tree's file
  count; the six files Round 24's scan
  omitted (.github/workflows/ci.yml,
  .gitignore, Cargo.lock, LICENSE,
  scripts/codepoint_scan.py,
  tests/fixtures/judgments_synthetic.json)
  stand in the list by name, and no file
  is excluded. The scan's planted
  control convicted inside each gate log
  (control 1 and 1), totals 0 and 0 at
  both trees. The retention form stands
  in its qualified final form: the gate
  scripts bank each launch's output
  under its own launch-numbered file
  (gate_base_launch1.log and
  gate_base_launch2.log at the base,
  gate_head_launch2.log at the head), no
  launch's log overwrites another's, and
  a launch that reaches its gate banks
  its output under its own name even
  when it counts nowhere; a launch that
  dies before its gate banks no log, and
  what stands for it is its named status
  evidence. Holds and launches: the base
  chain's launch 1 queued on the build
  slot for about ten minutes, acquired
  at 05:59:03Z, and its verify stage
  exited 1 on a lane instrument defect
  (the base tree SHA corrupted in
  verify_base30.py's tree URL by this
  lane's adaptation, a hybrid the API
  refused at 422, so the checkout's blob
  verification never ran at that
  launch); its build, base gate, census,
  and pin stages ran whole and banked
  under their own names
  (gate_base_launch1.log whole at the
  same 47 in the split, its census bank
  standing at census29_l1.log under the
  chain script's unadapted log name as
  written at that launch, its content
  the CENSUS30 run at ALL OK, the pin
  suite at 241 across 19), and the
  launch stands counted as failed at its
  verify stage, no figure of this round
  resting on it as the chain of record.
  Launch 2 queued for about thirty-seven
  minutes, acquired at 06:38:28Z, and
  ran the chain whole after the script's
  repair (verify, build, base gate,
  census, and the pin suite, every stage
  at exit 0), its banks the standing
  ones. The head gate's launch 1 died
  queued on the build slot in a runtime
  restart, its status file never
  created, no stage run and no bank
  created, counted dead on the lane
  record and relaunched in the same
  sitting; its launch 2 ran against the
  head checkout rebuilt from the pushed
  closure tree and verified file by
  file, and passed whole, its log the
  standing one. No other gate launch
  died or failed.
- VL-198. Fresh surface work, proposed
  in the lane record before it ran. The
  signed baselines, a twin surface
  Rounds 14 through 29 did not measure,
  thirty-one cells on genuinely
  independent arms
  (signbase30_results_l1.txt), standing
  at 31 of 31 agree, 0 failed, 0
  blocked, on its first launch with no
  amendment. Every earlier surface
  opened on non-negative baselines (the
  zero baseline celled at VL-144 and
  VL-150, negative values recorded
  verbatim as measure forms at VL-174
  but never as judged baselines), and
  this one cells the suggestion rule's
  signed path, the derivation read from
  the shipped source before running (the
  adverse difference taken in the
  direction's own sign and divided by
  the absolute value of the baseline,
  strictly greater than the margin in
  force, the zero baseline leaving the
  margin unconsulted). One refused open
  bounds the surface on the empty
  registry, at exit 1 on both arms with
  stderr byte-equal to its contracted
  line and the registry absent on both
  arms after it: an event margin of -5
  offered through the margin argument,
  the event margin's own negative form
  excluded by VL-192's design and
  uncelled until now, its refusal
  issuing from the defect check in both
  arms and not from either argument
  parser. The accepted writes stand on
  both arms' own registries: exp-s-a,
  cost judged lower over a baseline of
  -100 and rested at -80, closes rerun
  at (1, 2) with cost regressed at
  twenty percent past the default;
  exp-s-b, the same open rested at -120,
  closes keep at (4, 5), the difference
  standing at -20 and no measure
  regressed; exp-s-c, rescue judged
  higher over a baseline of -80 and
  rested at -100, closes rerun at (7, 8)
  with rescue regressed at twenty-five
  percent; exp-s-d, the same open rested
  at -60, closes keep at (10, 11);
  exp-s-e, cost lower over -100 and
  rescue higher over -50, rested at -70
  and -75, closes kill at (13, 14) with
  both measures regressed in judged
  order at thirty and fifty percent;
  exp-s-f, cost lower over -200 under an
  entry margin of 20 and rested at -160,
  closes keep at (16, 17), the move of
  exactly twenty percent standing at the
  margin and not past it; exp-s-g, the
  same open rested at -159, closes rerun
  at (19, 20) with cost regressed at
  twenty and a half percent; exp-s-h,
  cost lower over a float baseline of
  -2.5 and rested at -2.0, closes rerun
  at (22, 23) through the float
  pipeline; and exp-s-i, both measures
  judged over negative baselines and
  rested carrying cost alone at -100,
  closes keep at (25, 26) with rescue
  standing unmeasured in judged order.
  The show of exp-s-e returns its kill
  close on both arms, the list shows the
  nine experiments in open order with
  every verdict agreeing with its
  suggestion, check exits 0 on both arms
  reading the registry sound, and the
  registry stands at twenty-seven lines
  on both arms. Every derivation stood
  probed at this lane's hand against
  Round 29's chain binary and this
  round's oracle extraction, the source
  trees identical at this round's base,
  before the driver was written: all
  nine experiments close with the
  contracted figures on both arms, and
  the negative event margin refuses
  byte-equal on both arms with no
  registry created.
- VL-199. Storefront item 3 stands
  PASSING, verified again and not
  repaired: the description of 163
  characters, "An experiment registry
  for a fleet's setup. Every proposed
  change becomes a tracked experiment
  with a hypothesis, a run,
  measurements, and a verdict, keep or
  kill.", stands byte-identical in the
  repository description, in the
  Cargo.toml description field, and in
  the README lede, the match re-verified
  at this round's census against the
  base tree's own files and the live
  repository description.
- VL-200. Closure. README moves to 30 of
  100 in the closure commit only; slice
  and closure staged in separate
  directories (stage_slice,
  stage_closure); CONTRACT append-only
  in the standing sense, the prior
  rounds' text standing intact as a
  prefix and this round's own additions
  amended in place only where a repair's
  named accounting requires it; the tree
  diff base to head touches exactly
  CONTRACT.md and README.md. The round's
  weather, by name: the base chain's
  launch 1 failed at its verify stage on
  this lane's instrument defect, the
  base tree SHA corrupted in
  verify_base30.py's tree URL by the
  lane's own adaptation and refused by
  the API at 422, its remaining stages
  running whole and banked under their
  own names, its census bank standing at
  census29_l1.log under the chain
  script's unadapted log name as written
  at that launch, the launch counted and
  superseded by launch 2 after the
  script's repair; the build slot's
  queues, the base chain's launch 1
  waiting about ten minutes and its
  launch 2 about thirty-seven minutes
  behind the fleet's holds; the head
  gate's launch 1 dead queued on the
  build slot in a runtime restart, its
  status file never created, counted
  dead and relaunched in the same
  sitting, its launch 2 the standing
  one; the measurements and the fresh
  surface clean at their first launch;
  the extractions landed whole on their
  first passes. The API pool stood at
  3,721 remaining at the seating probe,
  3,519 after the read-back batch, 2,751
  at the first chain launch's verify
  probe, and reset by the second
  launch's probe at 4,188; no rate-limit
  stop hit this lane. No launch's
  failure is counted as evidence
  anywhere in this round's items.
