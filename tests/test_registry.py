"""VL-5 to VL-12. The registry."""
import json
from pathlib import Path

import pytest

from verdictloop.cli import main
from verdictloop.registry import (RegistryError, add_result,
                                  close_experiment, list_view,
                                  load_registry, open_experiment,
                                  show_view)

OPENED = {"type": "opened", "experiment": "exp-abc123",
          "at": "2026-01-05T09:00:00Z", "source": "retroforge",
          "proposal": "rules-edit-1",
          "baseline": "cost and rescue over the week before",
          "baseline_measures": {"cost": 100.0, "rescue": 80.0},
          "judged": [{"name": "cost", "direction": "lower"},
                     {"name": "rescue", "direction": "higher"}],
          "margin_pct": 10}


def opened(tmp_path, **overrides):
    path = tmp_path / "registry.jsonl"
    open_experiment(path, {**OPENED, **overrides})
    return path


def result_event(measures, at="2026-01-12T09:00:00Z"):
    return {"type": "result", "experiment": "exp-abc123",
            "at": at, "measures": measures}


def test_open_result_close_with_agreement(tmp_path):
    path = opened(tmp_path)
    add_result(path, result_event({"cost": 95.0, "rescue": 82.0}))
    closed = close_experiment(path, {
        "type": "closed", "experiment": "exp-abc123",
        "at": "2026-01-13T09:00:00Z", "verdict": "keep",
        "rationale": "both measures held"})
    assert closed["suggested"] == "keep"
    assert closed["regressed"] == []
    assert closed["rests_on"] == {"baseline": 1, "result": 2}
    view = list_view(path)
    assert view["experiments"] == [{
        "experiment": "exp-abc123", "source": "retroforge",
        "proposal": "rules-edit-1", "state": "closed",
        "results": 1, "verdict": "keep", "suggested": "keep",
        "agrees": True}]


def test_the_suggestion_kills_at_two_regressions(tmp_path):
    path = opened(tmp_path)
    add_result(path, result_event({"cost": 130.0, "rescue": 60.0}))
    closed = close_experiment(path, {
        "type": "closed", "experiment": "exp-abc123",
        "at": "2026-01-13T09:00:00Z", "verdict": "keep",
        "rationale": "the operator disagrees on the record"})
    assert closed["suggested"] == "kill"
    assert closed["regressed"] == ["cost", "rescue"]
    entry = list_view(path)["experiments"][0]
    assert entry["verdict"] == "keep"
    assert entry["suggested"] == "kill"
    assert entry["agrees"] is False


def test_one_regression_suggests_rerun_and_unmeasured_is_named(tmp_path):
    path = opened(tmp_path)
    add_result(path, result_event({"cost": 125.0}))
    closed = close_experiment(path, {
        "type": "closed", "experiment": "exp-abc123",
        "at": "2026-01-13T09:00:00Z", "verdict": "rerun",
        "rationale": "cost moved, rescue unknown",
        "successor": "exp-def456"})
    assert closed["suggested"] == "rerun"
    assert closed["regressed"] == ["cost"]
    assert closed["unmeasured"] == ["rescue"]
    assert closed["successor"] == "exp-def456"


def test_refusals_at_write_time(tmp_path):
    path = opened(tmp_path)
    with pytest.raises(RegistryError, match="already open"):
        open_experiment(path, dict(OPENED))
    with pytest.raises(RegistryError, match="not open"):
        add_result(path, {**result_event({"cost": 1.0}),
                          "experiment": "exp-ghost"})
    with pytest.raises(RegistryError, match="no result"):
        close_experiment(path, {
            "type": "closed", "experiment": "exp-abc123",
            "at": "2026-01-13T09:00:00Z", "verdict": "keep",
            "rationale": "too soon"})
    add_result(path, result_event({"cost": 95.0, "rescue": 82.0}))
    close_experiment(path, {
        "type": "closed", "experiment": "exp-abc123",
        "at": "2026-01-13T09:00:00Z", "verdict": "keep",
        "rationale": "held"})
    with pytest.raises(RegistryError, match="already closed"):
        add_result(path, result_event({"cost": 1.0},
                                      at="2026-01-14T09:00:00Z"))


def test_judged_names_must_exist_in_the_baseline(tmp_path):
    path = tmp_path / "registry.jsonl"
    bad = {**OPENED, "judged": [{"name": "luck",
                                 "direction": "lower"}]}
    with pytest.raises(RegistryError, match="not in the baseline"):
        open_experiment(path, bad)
    assert not path.exists()


def test_show_returns_the_full_history(tmp_path):
    path = opened(tmp_path)
    add_result(path, result_event({"cost": 95.0, "rescue": 82.0}))
    view = show_view(path, "exp-abc123")
    assert view["state"] == "resulted"
    assert view["opened"]["position"] == 1
    assert view["results"][0]["position"] == 2
    assert view["closed"] is None
    with pytest.raises(RegistryError, match="no experiment"):
        show_view(path, "exp-ghost")


def test_check_names_every_defect(tmp_path):
    path = tmp_path / "registry.jsonl"
    good = json.dumps(OPENED)
    lines = [good, "{not json",
             json.dumps(result_event({"cost": 1.0})),
             json.dumps(OPENED),
             json.dumps({"type": "result",
                         "experiment": "exp-ghost",
                         "at": "2026-01-12T09:00:00Z",
                         "measures": {"cost": 1.0}}),
             '{"type": "opened", "type": "opened"}']
    path.write_text("\n".join(lines) + "\n")
    _, defects = load_registry(path)
    text = "\n".join(defects)
    assert "line 2: not valid JSON" in text
    assert "duplicate opening" in text
    assert "unopened experiment exp-ghost" in text
    assert "line 6: not valid JSON" in text
    with pytest.raises(RegistryError, match="defects"):
        list_view(path)


def test_a_close_resting_on_a_wrong_position_is_a_defect(tmp_path):
    path = tmp_path / "registry.jsonl"
    forged = {"type": "closed", "experiment": "exp-abc123",
              "at": "2026-01-13T09:00:00Z", "verdict": "keep",
              "rationale": "forged", "suggested": "keep",
              "regressed": [], "unmeasured": [],
              "rests_on": {"baseline": 1, "result": 99}}
    lines = [json.dumps(OPENED),
             json.dumps(result_event({"cost": 1.0})),
             json.dumps(forged)]
    path.write_text("\n".join(lines) + "\n")
    _, defects = load_registry(path)
    assert any("rests on a result" in d for d in defects)


def test_the_cli_round_trip(tmp_path, capsys):
    path = str(tmp_path / "registry.jsonl")
    assert main(["open", path, "exp-abc123", "--source",
                 "retroforge", "--proposal", "rules-edit-1",
                 "--baseline", "the week before",
                 "--measures", '{"cost": 100, "rescue": 80}',
                 "--judged",
                 '[{"name": "cost", "direction": "lower"},'
                 ' {"name": "rescue", "direction": "higher"}]',
                 "--at", "2026-01-05T09:00:00Z"]) == 0
    assert main(["result", path, "exp-abc123", "--measures",
                 '{"cost": 90, "rescue": 85}',
                 "--at", "2026-01-12T09:00:00Z"]) == 0
    assert main(["close", path, "exp-abc123", "--verdict",
                 "keep", "--rationale", "held",
                 "--at", "2026-01-13T09:00:00Z"]) == 0
    assert main(["list", path]) == 0
    document = json.loads(capsys.readouterr().out)
    assert document["experiments"][0]["state"] == "closed"
    assert main(["check", path]) == 0
    assert "sound" in capsys.readouterr().out
    assert main(["result", path, "exp-abc123", "--measures",
                 '{"cost": 1}', "--at", "2026-01-14T09:00:00Z"]) == 1


def _close_line(suggested="keep", regressed=(), unmeasured=(),
                result=2, verdict="keep"):
    return {"type": "closed", "experiment": "exp-abc123",
            "at": "2026-01-13T09:00:00Z", "verdict": verdict,
            "rationale": "hand written", "suggested": suggested,
            "regressed": list(regressed),
            "unmeasured": list(unmeasured),
            "rests_on": {"baseline": 1, "result": result}}


def test_a_forged_suggestion_is_a_defect(tmp_path):
    path = tmp_path / "registry.jsonl"
    lines = [json.dumps(OPENED),
             json.dumps(result_event({"cost": 130.0,
                                      "rescue": 60.0})),
             json.dumps(_close_line())]
    path.write_text("\n".join(lines) + "\n")
    _, defects = load_registry(path)
    assert any("recorded suggestion does not match" in d
               for d in defects)


def test_a_close_on_a_stale_result_is_a_defect(tmp_path):
    path = tmp_path / "registry.jsonl"
    lines = [json.dumps(OPENED),
             json.dumps(result_event({"cost": 90.0,
                                      "rescue": 85.0})),
             json.dumps(result_event({"cost": 130.0,
                                      "rescue": 60.0},
                                     at="2026-01-12T10:00:00Z")),
             json.dumps(_close_line(result=2))]
    path.write_text("\n".join(lines) + "\n")
    _, defects = load_registry(path)
    assert any("not the current one" in d for d in defects)


def test_a_close_rests_on_the_latest_result_at_write(tmp_path):
    path = opened(tmp_path)
    add_result(path, result_event({"cost": 90.0, "rescue": 85.0}))
    add_result(path, result_event({"cost": 130.0, "rescue": 60.0},
                                  at="2026-01-12T10:00:00Z"))
    closed = close_experiment(path, {
        "type": "closed", "experiment": "exp-abc123",
        "at": "2026-01-13T09:00:00Z", "verdict": "kill",
        "rationale": "the second result settled it"})
    assert closed["rests_on"]["result"] == 3
    assert closed["suggested"] == "kill"


def test_duplicate_judged_names_and_negative_margins_refused(tmp_path):
    path = tmp_path / "registry.jsonl"
    dup = {**OPENED, "judged": [
        {"name": "cost", "direction": "lower"},
        {"name": "cost", "direction": "lower"},
        {"name": "rescue", "direction": "higher"}]}
    with pytest.raises(RegistryError, match="not unique"):
        open_experiment(path, dup)
    neg = {**OPENED, "margin_pct": -5}
    with pytest.raises(RegistryError, match="non-negative"):
        open_experiment(path, neg)


def test_a_successor_belongs_to_rerun_only(tmp_path):
    path = opened(tmp_path)
    add_result(path, result_event({"cost": 95.0, "rescue": 82.0}))
    with pytest.raises(RegistryError, match="rerun"):
        close_experiment(path, {
            "type": "closed", "experiment": "exp-abc123",
            "at": "2026-01-13T09:00:00Z", "verdict": "keep",
            "rationale": "held", "successor": "exp-next"})


def test_bare_dates_and_boolean_positions_are_defects(tmp_path):
    path = tmp_path / "registry.jsonl"
    bare = {**OPENED, "at": "2026-01-05"}
    path.write_text(json.dumps(bare) + "\n")
    _, defects = load_registry(path)
    assert any("at is not an ISO timestamp" in d for d in defects)
    forged = _close_line()
    forged["rests_on"] = {"baseline": True, "result": 2}
    path.write_text("\n".join([
        json.dumps(OPENED),
        json.dumps(result_event({"cost": 95.0, "rescue": 82.0})),
        json.dumps(forged)]) + "\n")
    _, defects = load_registry(path)
    assert any("rests_on does not name" in d for d in defects)


def test_stated_margins_govern_the_suggestion(tmp_path):
    path = tmp_path / "registry.jsonl"
    strict = {**OPENED, "margin_pct": 50}
    open_experiment(path, strict)
    add_result(path, result_event({"cost": 130.0, "rescue": 60.0}))
    closed = close_experiment(path, {
        "type": "closed", "experiment": "exp-abc123",
        "at": "2026-01-13T09:00:00Z", "verdict": "keep",
        "rationale": "inside the stated margins"})
    assert closed["suggested"] == "keep"
    per_entry = {**OPENED, "experiment": "exp-other",
                 "judged": [{"name": "cost", "direction": "lower",
                             "margin_pct": 5},
                            {"name": "rescue",
                             "direction": "higher"}]}
    path2 = tmp_path / "second.jsonl"
    open_experiment(path2, per_entry)
    add_result(path2, {**result_event({"cost": 108.0,
                                       "rescue": 82.0}),
                       "experiment": "exp-other"})
    closed2 = close_experiment(path2, {
        "type": "closed", "experiment": "exp-other",
        "at": "2026-01-13T09:00:00Z", "verdict": "rerun",
        "rationale": "the entry margin fired"})
    assert closed2["regressed"] == ["cost"]


def test_list_order_is_opening_order_and_appends_never_glue(tmp_path):
    path = tmp_path / "registry.jsonl"
    open_experiment(path, dict(OPENED))
    open_experiment(path, {**OPENED, "experiment": "exp-second"})
    add_result(path, {**result_event({"cost": 1.0}),
                      "experiment": "exp-second"})
    add_result(path, result_event({"cost": 1.0, "rescue": 80.0}))
    view = list_view(path)
    assert [e["experiment"] for e in view["experiments"]] == [
        "exp-abc123", "exp-second"]
    raw = path.read_bytes()
    path.write_bytes(raw.rstrip(b"\n"))
    add_result(path, result_event({"cost": 2.0, "rescue": 80.0},
                                  at="2026-01-13T09:00:00Z"))
    text = path.read_text()
    lines = text.strip().split("\n")
    assert len(lines) == 5
    _, defects = load_registry(path)
    assert defects == []


def test_a_view_never_overwrites_the_registry(tmp_path, capsys):
    path = opened(tmp_path)
    assert main(["list", str(path), "-o", str(path)]) == 1
    assert "never overwrites" in capsys.readouterr().err
    _, defects = load_registry(path)
    assert defects == []
