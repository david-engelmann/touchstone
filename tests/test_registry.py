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
