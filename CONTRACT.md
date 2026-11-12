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
