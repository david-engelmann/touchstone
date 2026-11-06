//! VL-20 to VL-26. Judgment in the contested band.
//! The combination rule is pinned with scripted
//! transports returning answers in Jev's recorded
//! shapes, the card rendering is pinned against a
//! hand-written text, the shipped fixtures replay,
//! and no test performs a network call.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use flighttape::identity::tape_digest;
use flighttape::json::parse;
use flighttape::value::{set_key, Text, Value};
use touchstone::emit::{dumps_line, Mode};
use touchstone::judge::{
    build_card, judge_card, FixtureTransport, JudgeError, LiveTransport, Transport,
};
use touchstone::registry::RegPath;
use touchstone::{
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

/// The canonical contested scenario: cost regressed
/// past its margin, rescue held, so the rule
/// suggests rerun and the author recorded keep.
const CARD: &str = "Experiment exp-abc123: rules-edit-1.\n\
cost: baseline 100.0, current 130.0, direction lower, margin 10 percent: regressed.\n\
rescue: baseline 80.0, current 82.0, direction higher, margin 10 percent: within margin.\n\
Regressed judged measures: cost.\n\
The author recorded verdict keep; the deterministic rule suggests rerun.";

fn ev(text: &str) -> Value {
    parse(text).expect("a test event parses")
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
        "touchstone-judge-{}-{}-{tag}",
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

/// Open the canonical experiment and record the
/// contested result; returns the registry path.
fn contested_registry(tag: &str) -> (PathBuf, RegPath) {
    let root = temp_root(tag);
    let path = reg(&root);
    open_experiment(&path, &opened()).expect("open");
    add_result(
        &path,
        &result_event(
            "{\"cost\": 130.0, \"rescue\": 82.0}",
            "2026-01-12T09:00:00Z",
        ),
    )
    .expect("result");
    (root, path)
}

fn get<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).expect("key present")
}

fn st(value: &Value) -> String {
    match value {
        Value::Str(text) => text.as_str().expect("well-formed").to_string(),
        _ => panic!("not a string"),
    }
}

fn obj(pairs: Vec<(&str, Value)>) -> Value {
    Value::Object(
        pairs
            .into_iter()
            .map(|(key, value)| (Text::from_str(key), value))
            .collect(),
    )
}

fn text(value: &str) -> Value {
    Value::Str(Text::from_str(value))
}

fn number(value: f64) -> Value {
    if value == value.trunc() && value.abs() < 9.0e15 {
        Value::Int((value as i64).to_string())
    } else {
        Value::Float(value)
    }
}

fn choice_response(choice: &str, confidence: f64) -> Value {
    obj(vec![
        ("model", text("jev-1.13.0")),
        (
            "answers",
            obj(vec![(
                "decision",
                obj(vec![
                    ("type", text("choice")),
                    ("choice", text(choice)),
                    ("confidence", number(confidence)),
                    (
                        "probabilities",
                        obj(vec![
                            ("keep", Value::Float(0.05)),
                            ("kill", Value::Float(0.1)),
                            ("rerun", Value::Float(0.8)),
                            ("abstain", Value::Float(0.05)),
                        ]),
                    ),
                ]),
            )]),
        ),
        (
            "usage",
            obj(vec![
                ("input_tokens", Value::Int("545".to_string())),
                ("output_tokens", Value::Int("50".to_string())),
            ]),
        ),
    ])
}

fn score_response(score: f64) -> Value {
    let pairs = |items: Vec<(String, Value)>| -> Value {
        Value::Object(
            items
                .into_iter()
                .map(|(k, v)| (Text::from_str(&k), v))
                .collect(),
        )
    };
    let legend: Vec<(String, Value)> = (0..10)
        .map(|i| (i.to_string(), text(&format!("Level {}", i + 1))))
        .collect();
    let probabilities: Vec<(String, Value)> = (0..10)
        .map(|i| (i.to_string(), Value::Float(0.1)))
        .collect();
    obj(vec![
        ("model", text("jev-1.13.0")),
        (
            "answers",
            obj(vec![(
                "decision",
                obj(vec![
                    ("type", text("score")),
                    ("score", number(score)),
                    ("confidence", Value::Float(0.5)),
                    ("legend", pairs(legend)),
                    ("probabilities", pairs(probabilities)),
                ]),
            )]),
        ),
        (
            "usage",
            obj(vec![
                ("input_tokens", Value::Int("480".to_string())),
                ("output_tokens", Value::Int("44".to_string())),
            ]),
        ),
    ])
}

struct Scripted {
    responses: HashMap<String, Value>,
    calls: RefCell<Vec<String>>,
}

impl Scripted {
    fn new(responses: HashMap<String, Value>) -> Scripted {
        Scripted {
            responses,
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl Transport for Scripted {
    fn ask(&self, _card: &str, kind: &str) -> Result<Value, JudgeError> {
        self.calls.borrow_mut().push(kind.to_string());
        Ok(self.responses.get(kind).expect("scripted kind").clone())
    }
}

struct Down;

impl Transport for Down {
    fn ask(&self, _card: &str, _kind: &str) -> Result<Value, JudgeError> {
        Err(JudgeError("the judge call failed".to_string()))
    }
}

struct Exploding;

impl Transport for Exploding {
    fn ask(&self, _card: &str, _kind: &str) -> Result<Value, JudgeError> {
        panic!("consulted outside the band");
    }
}

fn judged(choice: &str, confidence: f64) -> Scripted {
    Scripted::new(HashMap::from([
        ("choice".to_string(), choice_response(choice, confidence)),
        ("evidence".to_string(), score_response(7.5)),
        ("risk".to_string(), score_response(8.25)),
    ]))
}

#[test]
fn the_card_is_the_stated_text() {
    let card = build_card(
        &opened(),
        &result_event(
            "{\"cost\": 130.0, \"rescue\": 82.0}",
            "2026-01-12T09:00:00Z",
        ),
        "keep",
        "rerun",
        &[Text::from_str("cost")],
        &[],
    );
    assert_eq!(card, CARD);
    let expected = tape_digest(&text(CARD)).expect("digests");
    assert_eq!(expected.len(), 64);
}

#[test]
fn an_uncontested_close_never_consults() {
    let (_root, path) = contested_registry("agree");
    let closed = close_experiment(&path, &close_event("rerun", "rerun it"), Some(&Exploding))
        .expect("close");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "not_consulted");
    assert_eq!(
        st(get(judgment, "reason")),
        "the verdict and the suggestion agree"
    );
}

#[test]
fn a_contested_close_without_a_judge_closes_as_before() {
    let (_root, path) = contested_registry("noj");
    let closed = close_experiment(&path, &close_event("keep", "holding"), None).expect("close");
    assert_eq!(st(get(&closed, "verdict")), "keep");
    assert_eq!(st(get(&closed, "suggested")), "rerun");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "not_consulted");
    assert_eq!(st(get(judgment, "reason")), "no judge is configured");
}

#[test]
fn a_confident_choice_is_recorded_beside_the_verdict() {
    let (_root, path) = contested_registry("judged");
    let transport = judged("rerun", 0.9);
    let closed =
        close_experiment(&path, &close_event("keep", "holding"), Some(&transport)).expect("close");
    assert_eq!(
        *transport.calls.borrow(),
        vec![
            "choice".to_string(),
            "evidence".to_string(),
            "risk".to_string()
        ]
    );
    // The recorded verdict stands as the author's
    // act; the model's proposal stands beside it.
    assert_eq!(st(get(&closed, "verdict")), "keep");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "judged");
    assert_eq!(st(get(judgment, "model")), "jev-1.13.0");
    assert_eq!(st(get(judgment, "choice")), "rerun");
    assert_eq!(get(judgment, "confidence"), &Value::Float(0.9));
    assert_eq!(get(judgment, "evidence_strength"), &Value::Float(7.5));
    assert_eq!(get(judgment, "regression_risk"), &Value::Float(8.25));
    assert_eq!(
        get(get(judgment, "probabilities"), "rerun"),
        &Value::Float(0.8)
    );
    assert_eq!(st(get(judgment, "card")), CARD);
    let digest = tape_digest(&text(CARD)).expect("digests");
    assert_eq!(st(get(judgment, "card_sha256")), digest);
}

#[test]
fn the_judge_abstaining_refuses_the_close() {
    let (root, path) = contested_registry("abstain");
    let before = std::fs::read(root.join("registry.jsonl")).expect("registry");
    let error = close_experiment(
        &path,
        &close_event("keep", "holding"),
        Some(&judged("abstain", 0.9)),
    )
    .expect_err("refused");
    match error {
        RegError::Named(named) => {
            assert!(named.0.render_stderr().contains("the judge abstained"));
        }
        RegError::Crash => panic!("a named refusal, not the crash class"),
    }
    // Nothing was appended; the experiment is open.
    assert_eq!(std::fs::read(root.join("registry.jsonl")).unwrap(), before);
    let closed = close_experiment(&path, &close_event("keep", "holding"), None)
        .expect("the experiment is still open to close");
    assert_eq!(st(get(&closed, "verdict")), "keep");
}

#[test]
fn low_confidence_refuses_the_close() {
    let (root, path) = contested_registry("lowconf");
    let before = std::fs::read(root.join("registry.jsonl")).expect("registry");
    let error = close_experiment(
        &path,
        &close_event("keep", "holding"),
        Some(&judged("keep", 0.4)),
    )
    .expect_err("refused");
    match error {
        RegError::Named(named) => {
            let rendered = named.0.render_stderr();
            assert!(rendered.contains("below 0.6"), "{rendered}");
            assert!(rendered.contains("0.4"), "{rendered}");
        }
        RegError::Crash => panic!("a named refusal, not the crash class"),
    }
    assert_eq!(std::fs::read(root.join("registry.jsonl")).unwrap(), before);
}

#[test]
fn the_confidence_boundary_is_exact() {
    let (_root, path) = contested_registry("boundary-at");
    let closed = close_experiment(
        &path,
        &close_event("keep", "holding"),
        Some(&judged("keep", 0.6)),
    )
    .expect("0.6 is not below the floor");
    assert_eq!(st(get(get(&closed, "judgment"), "status")), "judged");
    let (_root, path) = contested_registry("boundary-under");
    let error = close_experiment(
        &path,
        &close_event("keep", "holding"),
        Some(&judged("keep", 0.59)),
    )
    .expect_err("0.59 is below the floor");
    assert!(matches!(error, RegError::Named(_)));
}

#[test]
fn an_unavailable_judge_still_closes() {
    let (_root, path) = contested_registry("down");
    let closed =
        close_experiment(&path, &close_event("keep", "holding"), Some(&Down)).expect("close");
    assert_eq!(st(get(&closed, "verdict")), "keep");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "unavailable");
    assert_eq!(st(get(judgment, "reason")), "the judge call failed");
    assert_eq!(st(get(judgment, "card")), CARD);
    // And the recorded close loads sound: check
    // re-derives the card and finds it consistent.
    let (_events, defects) = load_registry(&path).expect("loads");
    assert!(defects.is_empty(), "{defects:?}");
}

#[test]
fn malformed_answers_are_unavailable() {
    let mut missing_label = choice_response("rerun", 0.9);
    if let Some(Value::Object(probs)) = missing_label
        .get_mut("answers")
        .and_then(|a| a.get_mut("decision"))
        .and_then(|d| d.get_mut("probabilities"))
    {
        probs.retain(|(key, _)| key.as_str().as_deref() != Some("abstain"));
    }
    let transport = Scripted::new(HashMap::from([
        ("choice".to_string(), missing_label),
        ("evidence".to_string(), score_response(7.5)),
        ("risk".to_string(), score_response(8.25)),
    ]));
    let error = judge_card(CARD, &transport).expect_err("malformed");
    assert!(error.0.contains("malformed"), "{error}");

    let mut bad_score = score_response(7.5);
    if let Some(Value::Object(fields)) = bad_score
        .get_mut("answers")
        .and_then(|a| a.get_mut("decision"))
    {
        for (key, value) in fields.iter_mut() {
            if key.as_str().as_deref() == Some("score") {
                *value = text("high");
            }
        }
    }
    let transport = Scripted::new(HashMap::from([
        ("choice".to_string(), choice_response("rerun", 0.9)),
        ("evidence".to_string(), bad_score),
        ("risk".to_string(), score_response(8.25)),
    ]));
    let error = judge_card(CARD, &transport).expect_err("malformed");
    assert!(error.0.contains("malformed"), "{error}");

    let mut no_model = choice_response("rerun", 0.9);
    if let Value::Object(fields) = &mut no_model {
        fields.retain(|(key, _)| key.as_str().as_deref() != Some("model"));
    }
    let transport = Scripted::new(HashMap::from([
        ("choice".to_string(), no_model),
        ("evidence".to_string(), score_response(7.5)),
        ("risk".to_string(), score_response(8.25)),
    ]));
    let error = judge_card(CARD, &transport).expect_err("unpinned");
    assert!(error.0.contains("unpinned model"), "{error}");

    // Through the close, a malformed answer lands
    // the close as unavailable, never judged.
    let (_root, path) = contested_registry("malformed-close");
    let mut missing_label = choice_response("rerun", 0.9);
    if let Some(Value::Object(probs)) = missing_label
        .get_mut("answers")
        .and_then(|a| a.get_mut("decision"))
        .and_then(|d| d.get_mut("probabilities"))
    {
        probs.retain(|(key, _)| key.as_str().as_deref() != Some("abstain"));
    }
    let transport = Scripted::new(HashMap::from([
        ("choice".to_string(), missing_label),
        ("evidence".to_string(), score_response(7.5)),
        ("risk".to_string(), score_response(8.25)),
    ]));
    let closed =
        close_experiment(&path, &close_event("keep", "holding"), Some(&transport)).expect("close");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "unavailable");
    assert_eq!(
        st(get(judgment, "reason")),
        "the judge's choice answer is malformed"
    );
}

#[test]
fn out_of_range_answers_are_unavailable() {
    let scripted = |choice: Value, evidence: Value, risk: Value| {
        Scripted::new(HashMap::from([
            ("choice".to_string(), choice),
            ("evidence".to_string(), evidence),
            ("risk".to_string(), risk),
        ]))
    };
    let nan_choice = ev("{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"choice\", \"choice\": \"rerun\", \"confidence\": NaN, \
         \"probabilities\": {\"keep\": 0.1, \"kill\": 0.1, \"rerun\": 0.7, \
         \"abstain\": 0.1}}}}");
    let error = judge_card(
        CARD,
        &scripted(nan_choice, score_response(7.5), score_response(8.25)),
    )
    .expect_err("nan malformed");
    assert!(error.0.contains("malformed"), "{error}");
    let big = ev("{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"choice\", \"choice\": \"rerun\", \"confidence\": 0.9, \
         \"probabilities\": {\"keep\": 5.0, \"kill\": 5.0, \"rerun\": 5.0, \
         \"abstain\": 5.0}}}}");
    let error = judge_card(
        CARD,
        &scripted(big, score_response(7.5), score_response(8.25)),
    )
    .expect_err("big malformed");
    assert!(error.0.contains("malformed"), "{error}");
    let inf_score = ev("{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"score\", \"score\": Infinity, \"confidence\": 0.5}}}");
    let error = judge_card(
        CARD,
        &scripted(
            choice_response("rerun", 0.9),
            inf_score,
            score_response(8.25),
        ),
    )
    .expect_err("inf malformed");
    assert!(error.0.contains("malformed"), "{error}");
    let wrong_model = ev(
        "{\"model\": \"gpt-fabricated-9000\", \"answers\": {\"decision\": {\
         \"type\": \"score\", \"score\": 5, \"confidence\": 0.5}}}",
    );
    let error = judge_card(
        CARD,
        &scripted(
            choice_response("rerun", 0.9),
            score_response(7.5),
            wrong_model,
        ),
    )
    .expect_err("wrong model");
    assert!(error.0.contains("unpinned model"), "{error}");
}

#[test]
fn deterministic_refusals_precede_any_call() {
    // No result yet: the close refuses on state,
    // and the judge is never consulted.
    let root = temp_root("precedence");
    let path = reg(&root);
    open_experiment(&path, &opened()).expect("open");
    let error = close_experiment(&path, &close_event("keep", "too soon"), Some(&Exploding))
        .expect_err("no result");
    match error {
        RegError::Named(named) => assert!(named
            .0
            .render_stderr()
            .contains("has no result to rest a verdict on")),
        RegError::Crash => panic!("a named refusal"),
    }
    // A malformed close refuses on its own defects.
    let (_root, path) = contested_registry("precedence2");
    let error = close_experiment(
        &path,
        &close_event("maybe", "not a verdict"),
        Some(&Exploding),
    )
    .expect_err("malformed close");
    match error {
        RegError::Named(named) => assert!(named
            .0
            .render_stderr()
            .contains("verdict is not one of keep, kill, rerun")),
        RegError::Crash => panic!("a named refusal"),
    }
}

/// Render a value the way a file would carry it.
fn dumps_value(value: &Value) -> String {
    let bytes = dumps_line(value, Mode::Strict).expect("the value renders");
    String::from_utf8(bytes)
        .expect("utf8")
        .trim_end()
        .to_string()
}

/// Escape a string for embedding in a JSON text.
fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

#[test]
fn the_fixture_transport_replays_by_digest() {
    let root = temp_root("fixture");
    let digest = tape_digest(&text(CARD)).unwrap();
    let path = root.join("judgments.json");
    let payload = format!(
        "{{\"model\": \"jev-1.13.0\", \"cards\": [{{\"card_sha256\": \"{digest}\", \"card\": \"{}\", \"responses\": {{\
         \"choice\": {}, \"evidence\": {}, \"risk\": {}}}}}]}}",
        json_escape(CARD),
        dumps_value(&choice_response("keep", 0.77)),
        dumps_value(&score_response(3.0)),
        dumps_value(&score_response(9.0))
    );
    std::fs::write(&path, payload).unwrap();
    let transport = FixtureTransport::load(&path, "judgments").unwrap();
    let answers = judge_card(CARD, &transport).unwrap();
    assert_eq!(st(get(&answers, "choice")), "keep");
    assert_eq!(
        get(&answers, "evidence_strength"),
        &Value::Int("3".to_string())
    );
    assert_eq!(
        get(&answers, "regression_risk"),
        &Value::Int("9".to_string())
    );
    let error = judge_card(&format!("{CARD}\n"), &transport).expect_err("not recorded");
    assert!(error.0.contains("not recorded"), "{error}");
    let broken = root.join("broken.json");
    std::fs::write(&broken, "not json").unwrap();
    let error = FixtureTransport::load(&broken, "broken").expect_err("bad json");
    assert!(error.0.contains("not valid JSON"), "{error}");
}

#[test]
fn the_judgments_file_is_validated_whole() {
    let root = temp_root("whole");
    let digest = tape_digest(&text(CARD)).unwrap();
    let entry = format!(
        "{{\"card_sha256\": \"{digest}\", \"responses\": {{\
         \"choice\": {}, \"evidence\": {}, \"risk\": {}}}}}",
        dumps_value(&choice_response("rerun", 0.9)),
        dumps_value(&score_response(7.5)),
        dumps_value(&score_response(8.25))
    );
    let path = root.join("judgments.json");
    std::fs::write(&path, format!("{{\"cards\": [{entry}, {entry}]}}")).unwrap();
    let error = FixtureTransport::load(&path, "j").expect_err("twice");
    assert!(error.0.contains("twice"), "{error}");
    std::fs::write(
        &path,
        format!(
            "{{\"cards\": [{{\"card_sha256\": \"{digest}\", \"card\": \"{} \", \
             \"responses\": {{\"choice\": {}, \"evidence\": {}, \"risk\": {}}}}}]}}",
            json_escape(CARD),
            dumps_value(&choice_response("rerun", 0.9)),
            dumps_value(&score_response(7.5)),
            dumps_value(&score_response(8.25))
        ),
    )
    .unwrap();
    let error = FixtureTransport::load(&path, "j").expect_err("digest mismatch");
    assert!(error.0.contains("does not digest"), "{error}");
    std::fs::write(
        &path,
        format!("{{\"cards\": [{{\"card_sha256\": \"{digest}\"}}]}}"),
    )
    .unwrap();
    let error = FixtureTransport::load(&path, "j").expect_err("no responses");
    assert!(error.0.contains("no responses"), "{error}");
}

#[test]
fn the_shipped_fixtures_replay() {
    let shipped =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/judgments_synthetic.json");
    let data = parse(&std::fs::read_to_string(&shipped).unwrap()).unwrap();
    assert_eq!(st(get(&data, "model")), "jev-1.13.0");
    let cards = get(&data, "cards").as_array().unwrap();
    assert_eq!(cards.len(), 10);
    for entry in cards {
        let digest = tape_digest(entry.get("card").expect("card text")).unwrap();
        assert_eq!(st(get(entry, "card_sha256")), digest);
        let responses = get(entry, "responses").as_object().unwrap();
        assert_eq!(responses.len(), 3);
    }
    let transport = FixtureTransport::load(&shipped, "shipped").unwrap();
    // The first recorded card, replayed: the model
    // proposed the suggestion's verdict, rerun, at
    // 0.79, inside the recording window.
    let first = &cards[0];
    let answers = judge_card(&st(get(first, "card")), &transport).unwrap();
    assert_eq!(st(get(&answers, "model")), "jev-1.13.0");
    assert_eq!(st(get(&answers, "choice")), "rerun");
    assert_eq!(get(&answers, "confidence"), &Value::Float(0.79));
}

/// Open and result one of the recorded scenarios,
/// shaped exactly as the recording window shaped
/// it (the CLI's float margin), and return the
/// registry path.
fn recorded_registry(
    tag: &str,
    experiment: &str,
    proposal: &str,
    baseline: &str,
    judged_json: &str,
    result: &str,
) -> (PathBuf, RegPath) {
    let root = temp_root(tag);
    let path = reg(&root);
    let opened = ev(&format!(
        r#"{{"type": "opened", "experiment": "{experiment}",
          "at": "2026-01-05T09:00:00Z", "source": "retroforge",
          "proposal": "{proposal}",
          "baseline": "the week before",
          "baseline_measures": {baseline},
          "judged": {judged_json},
          "margin_pct": 10.0}}"#
    ));
    open_experiment(&path, &opened).expect("open");
    let result_event = ev(&format!(
        r#"{{"type": "result", "experiment": "{experiment}", "at": "2026-01-12T09:00:00Z", "measures": {result}}}"#
    ));
    add_result(&path, &result_event).expect("result");
    (root, path)
}

fn shipped_transport() -> FixtureTransport {
    let shipped =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/judgments_synthetic.json");
    FixtureTransport::load(&shipped, "shipped").unwrap()
}

#[test]
fn a_shipped_card_closes_an_experiment_judged() {
    let (_root, path) = recorded_registry(
        "ship001",
        "exp-vl-001",
        "rules-edit-1",
        "{\"cost\": 100.0, \"rescue\": 80.0}",
        "[{\"name\": \"cost\", \"direction\": \"lower\"}, {\"name\": \"rescue\", \"direction\": \"higher\"}]",
        "{\"cost\": 130.0, \"rescue\": 82.0}",
    );
    let close = ev(
        r#"{"type": "closed", "experiment": "exp-vl-001", "at": "2026-01-13T09:00:00Z", "verdict": "keep", "rationale": "the recorded position"}"#,
    );
    let transport = shipped_transport();
    let closed = close_experiment(&path, &close, Some(&transport)).expect("close");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "judged");
    assert_eq!(st(get(judgment, "choice")), "rerun");
    assert_eq!(get(judgment, "confidence"), &Value::Float(0.79));
    assert_eq!(st(get(&closed, "verdict")), "keep");
    let (_events, defects) = load_registry(&path).expect("loads");
    assert!(defects.is_empty(), "{defects:?}");
}

#[test]
fn a_shipped_below_floor_card_refuses_the_close() {
    // The recording window's second card: the model
    // proposed rerun at 0.53, under the floor, so
    // this close does not land.
    let (root, path) = recorded_registry(
        "ship002",
        "exp-vl-002",
        "prompt-tightening-2",
        "{\"cost\": 200.0}",
        "[{\"name\": \"cost\", \"direction\": \"lower\"}]",
        "{\"cost\": 230.0}",
    );
    let close = ev(
        r#"{"type": "closed", "experiment": "exp-vl-002", "at": "2026-01-13T09:00:00Z", "verdict": "kill", "rationale": "the recorded position"}"#,
    );
    let transport = shipped_transport();
    let error = close_experiment(&path, &close, Some(&transport)).expect_err("refused");
    match error {
        RegError::Named(named) => {
            let rendered = named.0.render_stderr();
            assert!(rendered.contains("below 0.6"), "{rendered}");
            assert!(rendered.contains("0.53"), "{rendered}");
        }
        RegError::Crash => panic!("a named refusal"),
    }
    let lines = std::fs::read_to_string(root.join("registry.jsonl")).unwrap();
    assert_eq!(lines.lines().count(), 2);
}

#[test]
fn a_nan_fixture_closes_unavailable_and_stays_json() {
    let root = temp_root("nan-fixture");
    let digest = tape_digest(&text(CARD)).unwrap();
    let path = root.join("judgments.json");
    let nan_choice = ev("{\"model\": \"jev-1.13.0\", \"answers\": {\"decision\": {\
         \"type\": \"choice\", \"choice\": \"rerun\", \"confidence\": NaN, \
         \"probabilities\": {\"keep\": 0.1, \"kill\": 0.1, \"rerun\": 0.7, \
         \"abstain\": 0.1}}}}");
    let payload = dumps_value(&obj(vec![(
        "cards",
        Value::Array(vec![obj(vec![
            ("card_sha256", text(&digest)),
            (
                "responses",
                obj(vec![
                    ("choice", nan_choice),
                    ("evidence", score_response(7.5)),
                    ("risk", score_response(8.25)),
                ]),
            ),
        ])]),
    )]));
    std::fs::write(&path, &payload).unwrap();
    assert!(payload.contains("NaN"));
    let (_root, reg_path) = contested_registry("nan-close");
    let transport = FixtureTransport::load(&path, "judgments").unwrap();
    let closed = close_experiment(&reg_path, &close_event("keep", "holding"), Some(&transport))
        .expect("close");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "unavailable");
    // The appended line is strict JSON: it carries
    // no NaN or Infinity token.
    let line = dumps_line(&closed, Mode::Strict).expect("the close renders");
    let rendered = String::from_utf8(line).unwrap();
    assert!(!rendered.contains("NaN"), "{rendered}");
    assert!(!rendered.contains("Infinity"), "{rendered}");
}

/// Close the canonical experiment with the scripted
/// judge and return (root, path, closed event).
fn judged_registry(tag: &str) -> (PathBuf, RegPath, Value) {
    let (root, path) = contested_registry(tag);
    let closed = close_experiment(
        &path,
        &close_event("keep", "holding"),
        Some(&judged("rerun", 0.9)),
    )
    .expect("close");
    (root, path, closed)
}

/// Rewrite the registry's closed line with a
/// mutated judgment section and return the defect
/// texts the load reports.
fn defects_after_rewrite(
    root: &Path,
    path: &RegPath,
    closed: &Value,
    mutate: impl Fn(&mut Value),
) -> Vec<String> {
    let file = root.join("registry.jsonl");
    let text_in = std::fs::read_to_string(&file).unwrap();
    let mut lines: Vec<String> = text_in.lines().map(|line| line.to_string()).collect();
    let mut forged = closed.clone();
    let mut judgment = get(&forged, "judgment").clone();
    mutate(&mut judgment);
    set_key(&mut forged, "judgment", judgment);
    let rendered = dumps_line(&forged, Mode::Strict).expect("renders");
    lines[2] = String::from_utf8(rendered).unwrap();
    std::fs::write(&file, lines.join("\n") + "\n").unwrap();
    let (_events, defects) = load_registry(path).expect("loads");
    defects
        .iter()
        .map(|defect| defect.render_stderr())
        .collect()
}

fn defects_after_mutation(tag: &str, mutate: impl Fn(&mut Value)) -> Vec<String> {
    let (root, path, closed) = judged_registry(tag);
    defects_after_rewrite(&root, &path, &closed, mutate)
}

#[test]
fn check_names_forged_judgments() {
    // A digest that does not match the card.
    let defects = defects_after_mutation("forge-digest", |judgment| {
        set_key(
            judgment,
            "card_sha256",
            text("0000000000000000000000000000000000000000000000000000000000000000"),
        );
    });
    assert!(
        defects
            .iter()
            .any(|d| d.contains("does not match the card it rests on")),
        "{defects:?}"
    );
    // A card that does not rebuild from the measures.
    let defects = defects_after_mutation("forge-card", |judgment| {
        set_key(judgment, "card", text("a hand-written card"));
    });
    assert!(
        defects
            .iter()
            .any(|d| d.contains("does not match the card it rests on")),
        "{defects:?}"
    );
    // A judged choice of abstain: an abstained close
    // does not exist.
    let defects = defects_after_mutation("forge-abstain", |judgment| {
        set_key(judgment, "choice", text("abstain"));
    });
    assert!(
        defects
            .iter()
            .any(|d| d.contains("names a choice that is not a verdict")),
        "{defects:?}"
    );
    // A confidence under the floor on a landed close.
    let defects = defects_after_mutation("forge-floor", |judgment| {
        set_key(judgment, "confidence", Value::Float(0.5));
    });
    assert!(
        defects.iter().any(|d| d.contains("below the floor of 0.6")),
        "{defects:?}"
    );
    // An unpinned model.
    let defects = defects_after_mutation("forge-model", |judgment| {
        set_key(judgment, "model", text("jev-9.9.9"));
    });
    assert!(
        defects.iter().any(|d| d.contains("unpinned model")),
        "{defects:?}"
    );
}

#[test]
fn check_names_sections_outside_the_recorded_shape() {
    // The genuine judged section is the writer's
    // exact shape and loads sound (VL-24a).
    let (_root, path, _closed) = judged_registry("shape-genuine");
    let (_events, defects) = load_registry(&path).expect("loads");
    assert!(defects.is_empty(), "{defects:?}");
    // A judged section carrying fields the writer
    // never emits (the red team's A5: a reason and
    // a note riding along).
    let defects = defects_after_mutation("shape-judged-extra", |judgment| {
        set_key(judgment, "reason", text("the judge explained itself"));
        set_key(judgment, "note", text("trust me"));
    });
    assert!(
        defects
            .iter()
            .any(|d| d.contains("the judged section's fields are not the recorded fields")),
        "{defects:?}"
    );
    // A judged section with its fields reordered
    // (the red team's A5c) is outside the shape
    // too: the fold holds not_consulted to exact
    // equality, order included, and VL-24a holds
    // the consulted sections to the same standard.
    let defects = defects_after_mutation("shape-judged-reordered", |judgment| {
        let pairs = match judgment {
            Value::Object(pairs) => pairs.clone(),
            _ => unreachable!("a judgment section is an object"),
        };
        *judgment = Value::Object(pairs.into_iter().rev().collect());
    });
    assert!(
        defects
            .iter()
            .any(|d| d.contains("the judged section's fields are not the recorded fields")),
        "{defects:?}"
    );
    // An unavailable section carrying a field the
    // writer never emits (the red team's A5b).
    let (root, path) = contested_registry("shape-unavailable");
    let closed =
        close_experiment(&path, &close_event("keep", "holding"), Some(&Down)).expect("close");
    assert_eq!(st(get(get(&closed, "judgment"), "status")), "unavailable");
    let defects = defects_after_rewrite(&root, &path, &closed, |judgment| {
        set_key(judgment, "note", text("smuggled"));
    });
    assert!(
        defects
            .iter()
            .any(|d| d.contains("the unavailable section's fields are not the recorded fields")),
        "{defects:?}"
    );
}

#[test]
fn a_non_object_answer_records_its_own_reason() {
    // A wholly non-object response names nothing:
    // the recorded unavailable reason is the guard
    // phrase, not a model failure (standards m1,
    // mirroring secondtake's answer_decision).
    let scripted = |choice: Value| {
        Scripted::new(HashMap::from([
            ("choice".to_string(), choice),
            ("evidence".to_string(), score_response(7.5)),
            ("risk".to_string(), score_response(8.25)),
        ]))
    };
    let error = judge_card(CARD, &scripted(Value::Array(vec![]))).expect_err("an array answer");
    assert_eq!(error.0, "the judge's answer is not an object");
    let error = judge_card(CARD, &scripted(text("a bare string"))).expect_err("a string answer");
    assert_eq!(error.0, "the judge's answer is not an object");
    // Through the close, the refusal lands as
    // unavailable carrying that reason.
    let (_root, path) = contested_registry("non-object-close");
    let closed = close_experiment(
        &path,
        &close_event("keep", "holding"),
        Some(&scripted(Value::Array(vec![]))),
    )
    .expect("close");
    let judgment = get(&closed, "judgment");
    assert_eq!(st(get(judgment, "status")), "unavailable");
    assert_eq!(
        st(get(judgment, "reason")),
        "the judge's answer is not an object"
    );
}

#[test]
fn check_names_a_judgment_outside_the_band() {
    // Take the agreeing close (verdict rerun, the
    // suggestion itself) and hang a judged section
    // on it: judgment enters only in the band.
    let (root, path) = contested_registry("forge-band");
    let closed = close_experiment(&path, &close_event("rerun", "rerun it"), None).expect("close");
    let (_jroot, _jpath, judged_close) = judged_registry("forge-band-src");
    let judgment = get(&judged_close, "judgment").clone();
    let mut forged = closed.clone();
    set_key(&mut forged, "judgment", judgment);
    let file = root.join("registry.jsonl");
    let text_in = std::fs::read_to_string(&file).unwrap();
    let mut lines: Vec<String> = text_in.lines().map(|line| line.to_string()).collect();
    let rendered = dumps_line(&forged, Mode::Strict).expect("renders");
    lines[2] = String::from_utf8(rendered).unwrap();
    std::fs::write(&file, lines.join("\n") + "\n").unwrap();
    let (_events, defects) = load_registry(&path).expect("loads");
    let rendered: Vec<String> = defects.iter().map(|d| d.render_stderr()).collect();
    assert!(
        rendered
            .iter()
            .any(|d| d.contains("where the verdict and the suggestion agree")),
        "{rendered:?}"
    );
}

#[test]
fn check_names_the_wrong_not_consulted_reason() {
    // An agreeing close whose section claims no
    // judge was configured: the fold knows the band
    // and expects the agreement reason.
    let (root, path) = contested_registry("forge-reason");
    let closed = close_experiment(&path, &close_event("rerun", "rerun it"), None).expect("close");
    let mut forged = closed.clone();
    set_key(
        &mut forged,
        "judgment",
        obj(vec![
            ("status", text("not_consulted")),
            ("reason", text("no judge is configured")),
        ]),
    );
    let file = root.join("registry.jsonl");
    let text_in = std::fs::read_to_string(&file).unwrap();
    let mut lines: Vec<String> = text_in.lines().map(|line| line.to_string()).collect();
    let rendered = dumps_line(&forged, Mode::Strict).expect("renders");
    lines[2] = String::from_utf8(rendered).unwrap();
    std::fs::write(&file, lines.join("\n") + "\n").unwrap();
    let (_events, defects) = load_registry(&path).expect("loads");
    let rendered: Vec<String> = defects.iter().map(|d| d.render_stderr()).collect();
    assert!(
        rendered
            .iter()
            .any(|d| d.contains("does not match the close it rests on")),
        "{rendered:?}"
    );
}

#[test]
fn the_views_carry_the_judgment() {
    let (_root, path, closed) = judged_registry("views");
    let view = list_view(&path).expect("list");
    let entry = &get(&view, "experiments").as_array().expect("entries")[0];
    let judgment = get(entry, "judgment");
    assert_eq!(st(get(judgment, "status")), "judged");
    assert_eq!(st(get(judgment, "choice")), "rerun");
    let shown = show_view(&path, &Text::from_str("exp-abc123")).expect("show");
    let shown_judgment = get(get(&shown, "closed"), "judgment");
    assert_eq!(shown_judgment, get(&closed, "judgment"));
    // A close recorded before the round carries no
    // section, and the views carry none either
    // (VL-26: where one exists).
    let root = temp_root("views-old");
    let path = reg(&root);
    let seeded = concat!(
        "{\"type\": \"opened\", \"experiment\": \"exp-old\", \"at\": \"2026-01-05T09:00:00Z\", \"source\": \"s\", \"proposal\": \"p\", \"baseline\": \"b\", \"baseline_measures\": {\"cost\": 100.0}, \"judged\": [{\"name\": \"cost\", \"direction\": \"lower\"}], \"margin_pct\": 10}\n",
        "{\"type\": \"result\", \"experiment\": \"exp-old\", \"at\": \"2026-01-12T09:00:00Z\", \"measures\": {\"cost\": 95.0}}\n",
        "{\"type\": \"closed\", \"experiment\": \"exp-old\", \"at\": \"2026-01-13T09:00:00Z\", \"verdict\": \"keep\", \"rationale\": \"held\", \"suggested\": \"keep\", \"regressed\": [], \"unmeasured\": [], \"rests_on\": {\"baseline\": 1, \"result\": 2}}\n",
    );
    std::fs::write(root.join("registry.jsonl"), seeded).unwrap();
    let view = list_view(&path).expect("list");
    let entry = &get(&view, "experiments").as_array().expect("entries")[0];
    assert!(entry.get("judgment").is_none());
    let shown = show_view(&path, &Text::from_str("exp-old")).expect("show");
    assert!(get(&shown, "closed").get("judgment").is_none());
}

#[test]
fn live_transport_names_undecodable_output() {
    let root = temp_root("live-decode");
    let stub = root.join("jev_stub.py");
    std::fs::write(
        &stub,
        "import sys\nsys.stdout.buffer.write(b'\\xff\\xfe')\n",
    )
    .unwrap();
    let transport = LiveTransport::new(std::ffi::OsString::from(stub.to_str().unwrap()));
    let error = transport.ask(CARD, "choice").expect_err("undecodable");
    assert!(error.0.contains("could not be read"), "{error}");
    let stub = root.join("jev_fail.py");
    std::fs::write(&stub, "import sys\nsys.exit(3)\n").unwrap();
    let transport = LiveTransport::new(std::ffi::OsString::from(stub.to_str().unwrap()));
    let error = transport.ask(CARD, "choice").expect_err("fails");
    assert!(error.0.contains("the judge call failed"), "{error}");
    let stub = root.join("jev_text.py");
    std::fs::write(&stub, "print('not json at all')\n").unwrap();
    let transport = LiveTransport::new(std::ffi::OsString::from(stub.to_str().unwrap()));
    let error = transport.ask(CARD, "choice").expect_err("not json");
    assert!(error.0.contains("not JSON"), "{error}");
}
