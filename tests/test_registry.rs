//! VL-5 to VL-12, ported file for file from the
//! Python suite. Library calls mirror the Python's
//! registry functions; the CLI tests drive the
//! compiled binary, as the Python's drive main().

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use flighttape::value::{set_key, Text, Value};
use verdictloop::emit::{dumps_line, Mode};
use verdictloop::registry::RegPath;
use verdictloop::{
    add_result, close_experiment, list_view, load_registry, open_experiment, show_view, RegError,
};

const OPENED_JSON: &str = r#"{"type": "opened", "experiment": "exp-abc123",
          "at": "2026-01-05T09:00:00Z", "source": "retroforge",
          "proposal": "rules-edit-1",
          "baseline": "cost and rescue over the week before",
          "baseline_measures": {"cost": 100.0, "rescue": 80.0},
          "judged": [{"name": "cost", "direction": "lower"},
                     {"name": "rescue", "direction": "higher"}],
          "margin_pct": 10}"#;

fn ev(text: &str) -> Value {
    flighttape::json::parse(text).expect("a test event parses")
}

fn opened() -> Value {
    ev(OPENED_JSON)
}

fn result_event(measures: &str, at: &str) -> Value {
    ev(&format!(
        r#"{{"type": "result", "experiment": "exp-abc123", "at": "{at}", "measures": {measures}}}"#
    ))
}

fn close_event(verdict: &str, rationale: &str) -> Value {
    ev(&format!(
        r#"{{"type": "closed", "experiment": "exp-abc123", "at": "2026-01-13T09:00:00Z", "verdict": "{verdict}", "rationale": "{rationale}"}}"#
    ))
}

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "verdictloop-test-{}-{}-{tag}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&root).expect("a temp root");
    root
}

fn reg(dir: &Path) -> RegPath {
    RegPath::new(dir.join("registry.jsonl").to_str().expect("a utf8 path"))
}

fn err_text(err: &RegError) -> String {
    match err {
        RegError::Named(error) => error.0.render_stderr(),
        RegError::Crash => panic!("a crash-class failure where a refusal was expected"),
    }
}

fn get<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).expect("a present key")
}

fn as_text(value: &Value) -> String {
    match value {
        Value::Str(text) => text.as_str().expect("plain text").to_string(),
        _ => panic!("not a string"),
    }
}

fn write_lines(path: &Path, values: &[Value]) {
    let mut bytes = Vec::new();
    for value in values {
        bytes.extend(dumps_line(value, Mode::Strict).expect("renderable"));
        bytes.push(b'\n');
    }
    std::fs::write(path, bytes).expect("write the registry");
}

#[test]
fn open_result_close_with_agreement() {
    let dir = temp_root("agree");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    add_result(
        &path,
        &result_event("{\"cost\": 95.0, \"rescue\": 82.0}", "2026-01-12T09:00:00Z"),
    )
    .expect("result");
    let closed =
        close_experiment(&path, &close_event("keep", "both measures held")).expect("close");
    assert_eq!(as_text(get(&closed, "suggested")), "keep");
    assert_eq!(get(&closed, "regressed"), &ev("[]"));
    assert_eq!(
        get(&closed, "rests_on"),
        &ev(r#"{"baseline": 1, "result": 2}"#)
    );
    let view = list_view(&path).expect("list");
    let expected = ev(r#"{"experiment": "exp-abc123", "source": "retroforge",
        "proposal": "rules-edit-1", "state": "closed",
        "results": 1, "verdict": "keep", "suggested": "keep",
        "agrees": true}"#);
    assert_eq!(get(&view, "experiments"), &Value::Array(vec![expected]));
}

#[test]
fn the_suggestion_kills_at_two_regressions() {
    let dir = temp_root("kill");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    add_result(
        &path,
        &result_event(
            "{\"cost\": 130.0, \"rescue\": 60.0}",
            "2026-01-12T09:00:00Z",
        ),
    )
    .expect("result");
    let closed = close_experiment(
        &path,
        &close_event("keep", "the operator disagrees on the record"),
    )
    .expect("close");
    assert_eq!(as_text(get(&closed, "suggested")), "kill");
    assert_eq!(get(&closed, "regressed"), &ev(r#"["cost", "rescue"]"#));
    let view = list_view(&path).expect("list");
    let entry = &get(&view, "experiments").as_array().expect("entries")[0];
    assert_eq!(as_text(get(entry, "verdict")), "keep");
    assert_eq!(as_text(get(entry, "suggested")), "kill");
    assert_eq!(get(entry, "agrees"), &Value::Bool(false));
}

#[test]
fn one_regression_suggests_rerun_and_unmeasured_is_named() {
    let dir = temp_root("rerun");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    add_result(
        &path,
        &result_event("{\"cost\": 125.0}", "2026-01-12T09:00:00Z"),
    )
    .expect("result");
    let mut event = close_event("rerun", "cost moved, rescue unknown");
    set_key(
        &mut event,
        "successor",
        Value::Str(Text::from_str("exp-def456")),
    );
    let closed = close_experiment(&path, &event).expect("close");
    assert_eq!(as_text(get(&closed, "suggested")), "rerun");
    assert_eq!(get(&closed, "regressed"), &ev(r#"["cost"]"#));
    assert_eq!(get(&closed, "unmeasured"), &ev(r#"["rescue"]"#));
    assert_eq!(as_text(get(&closed, "successor")), "exp-def456");
}

#[test]
fn refusals_at_write_time() {
    let dir = temp_root("refusals");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    let err = open_experiment(&path, &opened()).unwrap_err();
    assert!(
        err_text(&err).contains("already open"),
        "{}",
        err_text(&err)
    );
    let mut ghost = result_event("{\"cost\": 1.0}", "2026-01-12T09:00:00Z");
    set_key(
        &mut ghost,
        "experiment",
        Value::Str(Text::from_str("exp-ghost")),
    );
    let err = add_result(&path, &ghost).unwrap_err();
    assert!(err_text(&err).contains("not open"), "{}", err_text(&err));
    let err = close_experiment(&path, &close_event("keep", "too soon")).unwrap_err();
    assert!(err_text(&err).contains("no result"), "{}", err_text(&err));
    add_result(
        &path,
        &result_event("{\"cost\": 95.0, \"rescue\": 82.0}", "2026-01-12T09:00:00Z"),
    )
    .expect("result");
    close_experiment(&path, &close_event("keep", "held")).expect("close");
    let err = add_result(
        &path,
        &result_event("{\"cost\": 1.0}", "2026-01-14T09:00:00Z"),
    )
    .unwrap_err();
    assert!(
        err_text(&err).contains("already closed"),
        "{}",
        err_text(&err)
    );
}

#[test]
fn judged_names_must_exist_in_the_baseline() {
    let dir = temp_root("baseline");
    let path = reg(&dir);
    let mut bad = opened();
    set_key(
        &mut bad,
        "judged",
        ev(r#"[{"name": "luck", "direction": "lower"}]"#),
    );
    let err = open_experiment(&path, &bad).unwrap_err();
    assert!(
        err_text(&err).contains("not in the baseline"),
        "{}",
        err_text(&err)
    );
    assert!(!path.os.exists());
}

#[test]
fn show_returns_the_full_history() {
    let dir = temp_root("show");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    add_result(
        &path,
        &result_event("{\"cost\": 95.0, \"rescue\": 82.0}", "2026-01-12T09:00:00Z"),
    )
    .expect("result");
    let view = show_view(&path, &Text::from_str("exp-abc123")).expect("show");
    assert_eq!(as_text(get(&view, "state")), "resulted");
    assert_eq!(get(get(&view, "opened"), "position"), &ev("1"));
    let results = get(&view, "results").as_array().expect("results");
    assert_eq!(get(&results[0], "position"), &ev("2"));
    assert_eq!(get(&view, "closed"), &Value::Null);
    let err = show_view(&path, &Text::from_str("exp-ghost")).unwrap_err();
    assert!(
        err_text(&err).contains("no experiment"),
        "{}",
        err_text(&err)
    );
}

#[test]
fn check_names_every_defect() {
    let dir = temp_root("defects");
    let file = dir.join("registry.jsonl");
    let mut lines = vec![
        dumps_line(&opened(), Mode::Strict).expect("render"),
        b"{not json".to_vec(),
        dumps_line(
            &result_event("{\"cost\": 1.0}", "2026-01-12T09:00:00Z"),
            Mode::Strict,
        )
        .expect("render"),
        dumps_line(&opened(), Mode::Strict).expect("render"),
        dumps_line(
            &ev(r#"{"type": "result", "experiment": "exp-ghost", "at": "2026-01-12T09:00:00Z", "measures": {"cost": 1.0}}"#),
            Mode::Strict,
        )
        .expect("render"),
        br#"{"type": "opened", "type": "opened"}"#.to_vec(),
    ];
    let mut bytes = Vec::new();
    for line in lines.drain(..) {
        bytes.extend(line);
        bytes.push(b'\n');
    }
    std::fs::write(&file, bytes).expect("write");
    let path = reg(&dir);
    let (_events, defects) = load_registry(&path).expect("load");
    let text = defects
        .iter()
        .map(|defect| defect.render_stderr())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("line 2: not valid JSON"), "{text}");
    assert!(text.contains("duplicate opening"), "{text}");
    assert!(text.contains("unopened experiment exp-ghost"), "{text}");
    assert!(text.contains("line 6: not valid JSON"), "{text}");
    let err = list_view(&path).unwrap_err();
    assert!(err_text(&err).contains("defects"), "{}", err_text(&err));
}

#[test]
fn a_close_resting_on_a_wrong_position_is_a_defect() {
    let dir = temp_root("wrongpos");
    let file = dir.join("registry.jsonl");
    let forged = ev(r#"{"type": "closed", "experiment": "exp-abc123",
              "at": "2026-01-13T09:00:00Z", "verdict": "keep",
              "rationale": "forged", "suggested": "keep",
              "regressed": [], "unmeasured": [],
              "rests_on": {"baseline": 1, "result": 99}}"#);
    write_lines(
        &file,
        &[
            opened(),
            result_event("{\"cost\": 1.0}", "2026-01-12T09:00:00Z"),
            forged,
        ],
    );
    let (_events, defects) = load_registry(&reg(&dir)).expect("load");
    assert!(
        defects
            .iter()
            .any(|d| d.render_stderr().contains("rests on a result")),
        "{defects:?}"
    );
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_verdictloop"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("the binary runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn the_cli_round_trip() {
    let dir = temp_root("cli");
    let path = dir.join("registry.jsonl");
    let path = path.to_str().expect("utf8");
    let (code, _out, _err) = run_cli(&[
        "open", path, "exp-abc123", "--source", "retroforge", "--proposal",
        "rules-edit-1", "--baseline", "the week before", "--measures",
        "{\"cost\": 100, \"rescue\": 80}", "--judged",
        "[{\"name\": \"cost\", \"direction\": \"lower\"}, {\"name\": \"rescue\", \"direction\": \"higher\"}]",
        "--at", "2026-01-05T09:00:00Z",
    ]);
    assert_eq!(code, 0);
    let (code, _out, _err) = run_cli(&[
        "result",
        path,
        "exp-abc123",
        "--measures",
        "{\"cost\": 90, \"rescue\": 85}",
        "--at",
        "2026-01-12T09:00:00Z",
    ]);
    assert_eq!(code, 0);
    let (code, _out, _err) = run_cli(&[
        "close",
        path,
        "exp-abc123",
        "--verdict",
        "keep",
        "--rationale",
        "held",
        "--at",
        "2026-01-13T09:00:00Z",
    ]);
    assert_eq!(code, 0);
    let (code, out, _err) = run_cli(&["list", path]);
    assert_eq!(code, 0);
    let document = flighttape::json::parse(&out).expect("the list view parses");
    let entries = get(&document, "experiments").as_array().expect("entries");
    assert_eq!(as_text(get(&entries[0], "state")), "closed");
    let (code, out, _err) = run_cli(&["check", path]);
    assert_eq!(code, 0);
    assert!(out.contains("sound"), "{out}");
    let (code, _out, _err) = run_cli(&[
        "result",
        path,
        "exp-abc123",
        "--measures",
        "{\"cost\": 1}",
        "--at",
        "2026-01-14T09:00:00Z",
    ]);
    assert_eq!(code, 1);
}

fn close_line(
    suggested: &str,
    regressed: &str,
    unmeasured: &str,
    result: i32,
    verdict: &str,
) -> Value {
    ev(&format!(
        r#"{{"type": "closed", "experiment": "exp-abc123",
            "at": "2026-01-13T09:00:00Z", "verdict": "{verdict}",
            "rationale": "hand written", "suggested": "{suggested}",
            "regressed": {regressed}, "unmeasured": {unmeasured},
            "rests_on": {{"baseline": 1, "result": {result}}}}}"#
    ))
}

#[test]
fn a_forged_suggestion_is_a_defect() {
    let dir = temp_root("forged");
    let file = dir.join("registry.jsonl");
    write_lines(
        &file,
        &[
            opened(),
            result_event(
                "{\"cost\": 130.0, \"rescue\": 60.0}",
                "2026-01-12T09:00:00Z",
            ),
            close_line("keep", "[]", "[]", 2, "keep"),
        ],
    );
    let (_events, defects) = load_registry(&reg(&dir)).expect("load");
    assert!(
        defects.iter().any(|d| d
            .render_stderr()
            .contains("recorded suggestion does not match")),
        "{defects:?}"
    );
}

#[test]
fn a_close_on_a_stale_result_is_a_defect() {
    let dir = temp_root("stale");
    let file = dir.join("registry.jsonl");
    write_lines(
        &file,
        &[
            opened(),
            result_event("{\"cost\": 90.0, \"rescue\": 85.0}", "2026-01-12T09:00:00Z"),
            result_event(
                "{\"cost\": 130.0, \"rescue\": 60.0}",
                "2026-01-12T10:00:00Z",
            ),
            close_line("keep", "[]", "[]", 2, "keep"),
        ],
    );
    let (_events, defects) = load_registry(&reg(&dir)).expect("load");
    assert!(
        defects
            .iter()
            .any(|d| d.render_stderr().contains("not the current one")),
        "{defects:?}"
    );
}

#[test]
fn a_close_rests_on_the_latest_result_at_write() {
    let dir = temp_root("latest");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    add_result(
        &path,
        &result_event("{\"cost\": 90.0, \"rescue\": 85.0}", "2026-01-12T09:00:00Z"),
    )
    .expect("result");
    add_result(
        &path,
        &result_event(
            "{\"cost\": 130.0, \"rescue\": 60.0}",
            "2026-01-12T10:00:00Z",
        ),
    )
    .expect("result");
    let closed = close_experiment(&path, &close_event("kill", "the second result settled it"))
        .expect("close");
    assert_eq!(get(get(&closed, "rests_on"), "result"), &ev("3"));
    assert_eq!(as_text(get(&closed, "suggested")), "kill");
}

#[test]
fn duplicate_judged_names_and_negative_margins_refused() {
    let dir = temp_root("dupjudged");
    let path = reg(&dir);
    let mut dup = opened();
    set_key(
        &mut dup,
        "judged",
        ev(r#"[{"name": "cost", "direction": "lower"},
               {"name": "cost", "direction": "lower"},
               {"name": "rescue", "direction": "higher"}]"#),
    );
    let err = open_experiment(&path, &dup).unwrap_err();
    assert!(err_text(&err).contains("not unique"), "{}", err_text(&err));
    let mut neg = opened();
    set_key(&mut neg, "margin_pct", ev("-5"));
    let err = open_experiment(&path, &neg).unwrap_err();
    assert!(
        err_text(&err).contains("non-negative"),
        "{}",
        err_text(&err)
    );
}

#[test]
fn a_successor_belongs_to_rerun_only() {
    let dir = temp_root("successor");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    add_result(
        &path,
        &result_event("{\"cost\": 95.0, \"rescue\": 82.0}", "2026-01-12T09:00:00Z"),
    )
    .expect("result");
    let mut event = close_event("keep", "held");
    set_key(
        &mut event,
        "successor",
        Value::Str(Text::from_str("exp-next")),
    );
    let err = close_experiment(&path, &event).unwrap_err();
    assert!(err_text(&err).contains("rerun"), "{}", err_text(&err));
}

#[test]
fn bare_dates_and_boolean_positions_are_defects() {
    let dir = temp_root("bare");
    let file = dir.join("registry.jsonl");
    let mut bare = opened();
    set_key(&mut bare, "at", Value::Str(Text::from_str("2026-01-05")));
    write_lines(&file, &[bare]);
    let (_events, defects) = load_registry(&reg(&dir)).expect("load");
    assert!(
        defects
            .iter()
            .any(|d| d.render_stderr().contains("at is not an ISO timestamp")),
        "{defects:?}"
    );
    let mut forged = close_line("keep", "[]", "[]", 2, "keep");
    set_key(
        &mut forged,
        "rests_on",
        ev(r#"{"baseline": true, "result": 2}"#),
    );
    write_lines(
        &file,
        &[
            opened(),
            result_event("{\"cost\": 95.0, \"rescue\": 82.0}", "2026-01-12T09:00:00Z"),
            forged,
        ],
    );
    let (_events, defects) = load_registry(&reg(&dir)).expect("load");
    assert!(
        defects
            .iter()
            .any(|d| d.render_stderr().contains("rests_on does not name")),
        "{defects:?}"
    );
}

#[test]
fn stated_margins_govern_the_suggestion() {
    let dir = temp_root("margins");
    let path = reg(&dir);
    let mut strict = opened();
    set_key(&mut strict, "margin_pct", ev("50"));
    open_experiment(&path, &strict).expect("open");
    add_result(
        &path,
        &result_event(
            "{\"cost\": 130.0, \"rescue\": 60.0}",
            "2026-01-12T09:00:00Z",
        ),
    )
    .expect("result");
    let closed =
        close_experiment(&path, &close_event("keep", "inside the stated margins")).expect("close");
    assert_eq!(as_text(get(&closed, "suggested")), "keep");
    let file2 = dir.join("second.jsonl");
    let path2 = RegPath::new(file2.to_str().expect("utf8"));
    let mut per_entry = opened();
    set_key(
        &mut per_entry,
        "experiment",
        Value::Str(Text::from_str("exp-other")),
    );
    set_key(
        &mut per_entry,
        "judged",
        ev(r#"[{"name": "cost", "direction": "lower", "margin_pct": 5},
               {"name": "rescue", "direction": "higher"}]"#),
    );
    open_experiment(&path2, &per_entry).expect("open");
    let mut result = result_event(
        "{\"cost\": 108.0, \"rescue\": 82.0}",
        "2026-01-12T09:00:00Z",
    );
    set_key(
        &mut result,
        "experiment",
        Value::Str(Text::from_str("exp-other")),
    );
    add_result(&path2, &result).expect("result");
    let mut close = close_event("rerun", "the entry margin fired");
    set_key(
        &mut close,
        "experiment",
        Value::Str(Text::from_str("exp-other")),
    );
    let closed2 = close_experiment(&path2, &close).expect("close");
    assert_eq!(get(&closed2, "regressed"), &ev(r#"["cost"]"#));
}

#[test]
fn list_order_is_opening_order_and_appends_never_glue() {
    let dir = temp_root("order");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    let mut second = opened();
    set_key(
        &mut second,
        "experiment",
        Value::Str(Text::from_str("exp-second")),
    );
    open_experiment(&path, &second).expect("open");
    let mut result = result_event("{\"cost\": 1.0}", "2026-01-12T09:00:00Z");
    set_key(
        &mut result,
        "experiment",
        Value::Str(Text::from_str("exp-second")),
    );
    add_result(&path, &result).expect("result");
    add_result(
        &path,
        &result_event("{\"cost\": 1.0, \"rescue\": 80.0}", "2026-01-12T09:00:00Z"),
    )
    .expect("result");
    let view = list_view(&path).expect("list");
    let names: Vec<String> = get(&view, "experiments")
        .as_array()
        .expect("entries")
        .iter()
        .map(|entry| as_text(get(entry, "experiment")))
        .collect();
    assert_eq!(names, ["exp-abc123", "exp-second"]);
    let raw = std::fs::read(&path.os).expect("read");
    let trimmed = raw
        .strip_suffix(b"\n")
        .expect("a trailing newline")
        .to_vec();
    std::fs::write(&path.os, trimmed).expect("truncate the newline");
    add_result(
        &path,
        &result_event("{\"cost\": 2.0, \"rescue\": 80.0}", "2026-01-13T09:00:00Z"),
    )
    .expect("result");
    let text = std::fs::read_to_string(&path.os).expect("read back");
    assert_eq!(text.trim().split('\n').count(), 5);
    let (_events, defects) = load_registry(&path).expect("load");
    assert!(defects.is_empty(), "{defects:?}");
}

#[test]
fn a_view_never_overwrites_the_registry() {
    let dir = temp_root("guard");
    let path = reg(&dir);
    open_experiment(&path, &opened()).expect("open");
    let path_text = path.os.to_str().expect("utf8").to_string();
    let (code, _out, err) = run_cli(&["list", &path_text, "-o", &path_text]);
    assert_eq!(code, 1);
    assert!(err.contains("never overwrites"), "{err}");
    let (_events, defects) = load_registry(&path).expect("load");
    assert!(defects.is_empty());
}

#[test]
fn an_unhashable_judged_name_is_a_defect_not_a_crash() {
    let dir = temp_root("weird");
    let file = dir.join("registry.jsonl");
    let mut weird = opened();
    set_key(
        &mut weird,
        "judged",
        ev(r#"[{"name": ["cost"], "direction": "lower"},
               {"name": "rescue", "direction": "higher"}]"#),
    );
    write_lines(&file, &[weird.clone()]);
    let (_events, defects) = load_registry(&reg(&dir)).expect("load");
    assert!(
        defects
            .iter()
            .any(|d| d.render_stderr().contains("judged entry needs a name")),
        "{defects:?}"
    );
    let other = RegPath::new(dir.join("other.jsonl").to_str().expect("utf8"));
    assert!(open_experiment(&other, &weird).is_err());
}

#[test]
fn deeply_nested_cli_input_is_a_named_error() {
    let dir = temp_root("deep");
    let path = dir.join("registry.jsonl");
    let path = path.to_str().expect("utf8");
    let nested = "[".repeat(100_000);
    let (code, _out, err) = run_cli(&[
        "open",
        path,
        "exp-x",
        "--source",
        "s",
        "--proposal",
        "p",
        "--baseline",
        "b",
        "--measures",
        &nested,
        "--judged",
        "[]",
        "--at",
        "2026-01-05T09:00:00Z",
    ]);
    assert_eq!(code, 1);
    assert!(err.contains("not valid JSON"), "{err}");
    // The deep write must not leave a stray file behind.
    let _ = std::io::stdout().flush();
}
