"""VL-5 to VL-11. The registry: an append-only event
file, folded into state on every read."""
from __future__ import annotations

import json
import re
from datetime import datetime
from pathlib import Path

VERDICTS = ("keep", "kill", "rerun")
EVENT_TYPES = ("opened", "result", "closed")
DIRECTIONS = ("lower", "higher")


class RegistryError(ValueError):
    """A named refusal (family form, VL-12)."""


def _is_number(value) -> bool:
    return (isinstance(value, (int, float))
            and not isinstance(value, bool)
            and value == value
            and value not in (float("inf"), float("-inf")))


_TIMESTAMP_FORM = re.compile(
    r"^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}")


def _is_timestamp(value) -> bool:
    if not isinstance(value, str) \
            or not _TIMESTAMP_FORM.match(value):
        return False
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return False
    return True


def _no_duplicate_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate key {key!r}")
        result[key] = value
    return result


def _reject_constant(value):
    raise ValueError(f"invalid constant {value}")


def event_defects(event, position: int) -> list[str]:
    where = f"line {position}"
    if not isinstance(event, dict):
        return [f"{where}: the event is not an object"]
    defects = []
    kind = event.get("type")
    if kind not in EVENT_TYPES:
        defects.append(f"{where}: unknown event type {kind!r}")
    if not isinstance(event.get("experiment"), str) \
            or not event.get("experiment"):
        defects.append(f"{where}: experiment is not a "
                       f"non-empty string")
    if not _is_timestamp(event.get("at")):
        defects.append(f"{where}: at is not an ISO timestamp")
    if defects or kind not in EVENT_TYPES:
        return defects
    if kind == "opened":
        for field in ("source", "proposal", "baseline"):
            if not isinstance(event.get(field), str) \
                    or not event.get(field):
                defects.append(f"{where}: {field} is not a "
                               f"non-empty string")
        measures = event.get("baseline_measures")
        if not isinstance(measures, dict) or not measures:
            defects.append(f"{where}: baseline_measures is "
                           f"not a non-empty object")
        elif any(not _is_number(v) for v in measures.values()):
            defects.append(f"{where}: baseline_measures holds "
                           f"a value that is not a finite number")
        judged = event.get("judged")
        if not isinstance(judged, list) or not judged:
            defects.append(f"{where}: judged is not a "
                           f"non-empty list")
        else:
            names = [e.get("name") for e in judged
                     if isinstance(e, dict)]
            if len(names) != len(set(names)):
                defects.append(f"{where}: judged names are "
                               f"not unique")
            for entry in judged:
                if not isinstance(entry, dict) \
                        or not isinstance(entry.get("name"), str) \
                        or entry.get("direction") not in DIRECTIONS:
                    defects.append(
                        f"{where}: a judged entry needs a name "
                        f"and a direction of lower or higher")
                elif "margin_pct" in entry \
                        and (not _is_number(entry["margin_pct"])
                             or entry["margin_pct"] < 0):
                    defects.append(
                        f"{where}: a judged margin_pct is not "
                        f"a non-negative number")
                elif isinstance(measures, dict) \
                        and entry.get("name") not in measures:
                    defects.append(
                        f"{where}: judged measure "
                        f"{entry.get('name')!r} is not in the "
                        f"baseline measures")
        if "margin_pct" in event \
                and (not _is_number(event["margin_pct"])
                     or event["margin_pct"] < 0):
            defects.append(f"{where}: margin_pct is not a "
                           f"non-negative number")
    if kind == "result":
        measures = event.get("measures")
        if not isinstance(measures, dict) or not measures:
            defects.append(f"{where}: measures is not a "
                           f"non-empty object")
        elif any(not _is_number(v) for v in measures.values()):
            defects.append(f"{where}: measures holds a value "
                           f"that is not a finite number")
    if kind == "closed":
        if event.get("verdict") not in VERDICTS:
            defects.append(f"{where}: verdict is not one of "
                           f"keep, kill, rerun")
        if not isinstance(event.get("rationale"), str) \
                or not event.get("rationale"):
            defects.append(f"{where}: rationale is not a "
                           f"non-empty string")
        rests = event.get("rests_on")
        def _position(value):
            return isinstance(value, int) \
                and not isinstance(value, bool)
        if not isinstance(rests, dict) \
                or not _position(rests.get("baseline")) \
                or not _position(rests.get("result")):
            defects.append(f"{where}: rests_on does not name "
                           f"the baseline and result positions")
        if not isinstance(event.get("suggested"), str) \
                or event.get("suggested") not in VERDICTS:
            defects.append(f"{where}: the recorded suggestion "
                           f"is not a verdict")
        for field in ("regressed", "unmeasured"):
            if not isinstance(event.get(field), list) \
                    or any(not isinstance(n, str)
                           for n in event.get(field, [])):
                defects.append(f"{where}: {field} is not a "
                               f"list of measure names")
        if "successor" in event \
                and event.get("verdict") != "rerun":
            defects.append(f"{where}: a successor is recorded "
                           f"only on a rerun verdict")
    return defects


def load_registry(path: Path):
    """(events, defects): events carry their 1-based
    positions; defects names every bad line and every
    sequence violation (VL-5, VL-11, VL-11a)."""
    events: list[tuple[int, dict]] = []
    defects: list[str] = []
    try:
        raw = path.read_bytes()
    except FileNotFoundError:
        return [], [f"no such registry, {path}"]
    except OSError as exc:
        return [], [f"cannot read the registry at {path}: {exc}"]
    try:
        text = raw.decode("utf-8-sig")
    except UnicodeDecodeError:
        return [], ["the registry is not valid UTF-8"]
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    for position, line in enumerate(lines, start=1):
        if not line.strip():
            defects.append(f"line {position}: blank line")
            continue
        try:
            event = json.loads(
                line, object_pairs_hook=_no_duplicate_keys,
                parse_constant=_reject_constant)
        except (ValueError, RecursionError):
            defects.append(f"line {position}: not valid JSON")
            continue
        found = event_defects(event, position)
        if found:
            defects.extend(found)
            continue
        events.append((position, event))
    defects.extend(_fold_defects(events))
    return events, defects


def _fold_defects(events) -> list[str]:
    defects = []
    opened: dict[str, int] = {}
    closed: set[str] = set()
    positions: dict[int, tuple[str, str]] = {}
    for position, event in events:
        positions[position] = (event["experiment"], event["type"])
    for position, event in events:
        experiment, kind = event["experiment"], event["type"]
        if kind == "opened":
            if experiment in opened:
                defects.append(
                    f"line {position}: duplicate opening of "
                    f"{experiment}")
            else:
                opened[experiment] = position
        elif experiment not in opened:
            defects.append(
                f"line {position}: {kind} event for unopened "
                f"experiment {experiment}")
        elif kind == "result" and experiment in closed:
            defects.append(
                f"line {position}: result after the close of "
                f"{experiment}")
        elif kind == "closed":
            if experiment in closed:
                defects.append(
                    f"line {position}: second close of "
                    f"{experiment}")
            closed.add(experiment)
            rests = event.get("rests_on") or {}
            if rests.get("baseline") != opened.get(experiment):
                defects.append(
                    f"line {position}: the close rests on a "
                    f"baseline that is not the opening of "
                    f"{experiment}")
            result_at = rests.get("result")
            if positions.get(result_at) != (experiment, "result"):
                defects.append(
                    f"line {position}: the close rests on a "
                    f"result that is not a result of "
                    f"{experiment}")
            else:
                later = [pos for pos, other in events
                         if other["experiment"] == experiment
                         and other["type"] == "result"
                         and result_at < pos < position]
                if later:
                    defects.append(
                        f"line {position}: the close rests on "
                        f"a result that is not the current "
                        f"one; a later result stands at line "
                        f"{later[-1]}")
                else:
                    opened_event = dict(events)[opened[experiment]]
                    result_event = dict(events)[result_at]
                    found = suggested_verdict(opened_event,
                                              result_event)
                    recorded = (event.get("suggested"),
                                event.get("regressed"),
                                event.get("unmeasured"))
                    if recorded != found:
                        defects.append(
                            f"line {position}: the recorded "
                            f"suggestion does not match the "
                            f"measures it rests on")
    return defects


def fold(events) -> dict:
    """Experiment states in opening order (VL-10)."""
    states: dict[str, dict] = {}
    for position, event in events:
        experiment = event["experiment"]
        if event["type"] == "opened":
            states[experiment] = {
                "experiment": experiment, "opened": event,
                "opened_at_position": position, "results": [],
                "closed": None, "closed_at_position": None}
        elif event["type"] == "result":
            states[experiment]["results"].append((position, event))
        elif event["type"] == "closed":
            states[experiment]["closed"] = event
            states[experiment]["closed_at_position"] = position
    for state in states.values():
        if state["closed"] is not None:
            state["state"] = "closed"
        elif state["results"]:
            state["state"] = "resulted"
        else:
            state["state"] = "open"
    return states


def suggested_verdict(opened: dict, result: dict):
    """The VL-9 rule: (suggestion, regressed, unmeasured)."""
    baseline = opened["baseline_measures"]
    measures = result["measures"]
    default_margin = opened.get("margin_pct", 10)
    regressed, unmeasured = [], []
    for entry in opened["judged"]:
        name = entry["name"]
        if name not in measures:
            unmeasured.append(name)
            continue
        margin = entry.get("margin_pct", default_margin)
        base, value = baseline[name], measures[name]
        if entry["direction"] == "lower":
            bad = value > 0 if base == 0 else \
                (value - base) / abs(base) * 100 > margin
        else:
            bad = False if base == 0 else \
                (base - value) / abs(base) * 100 > margin
        if bad:
            regressed.append(name)
    suggestion = ("keep" if not regressed else
                  "kill" if len(regressed) >= 2 else "rerun")
    return suggestion, regressed, unmeasured


def _require_sound(path: Path):
    events, defects = load_registry(path)
    if defects:
        raise RegistryError(
            "the registry has defects: " + "; ".join(defects))
    return events


def _append(path: Path, event: dict) -> None:
    line = json.dumps(event, ensure_ascii=False) + "\n"
    try:
        prefix = ""
        if path.exists() and path.stat().st_size > 0:
            with open(path, "rb") as handle:
                handle.seek(-1, 2)
                if handle.read(1) != b"\n":
                    prefix = "\n"
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(prefix + line)
            handle.flush()
    except OSError as exc:
        raise RegistryError(
            f"cannot append to the registry at {path}: "
            f"{exc}") from exc


def open_experiment(path: Path, event: dict) -> None:
    events = _require_sound(path) if path.exists() else []
    defects = event_defects(event, _next_position(path))
    if defects:
        raise RegistryError("; ".join(defects))
    states = fold(events)
    if event["experiment"] in states:
        raise RegistryError(
            f"experiment {event['experiment']} is already open")
    _append(path, event)


def _next_position(path: Path) -> int:
    try:
        text = path.read_bytes().decode("utf-8-sig")
    except (OSError, UnicodeDecodeError):
        return 1
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    return len(lines) + 1


def add_result(path: Path, event: dict) -> None:
    events = _require_sound(path)
    defects = event_defects(event, _next_position(path))
    if defects:
        raise RegistryError("; ".join(defects))
    states = fold(events)
    state = states.get(event["experiment"])
    if state is None:
        raise RegistryError(
            f"experiment {event['experiment']} is not open")
    if state["closed"] is not None:
        raise RegistryError(
            f"experiment {event['experiment']} is already closed")
    _append(path, event)


def close_experiment(path: Path, event: dict) -> dict:
    events = _require_sound(path)
    states = fold(events)
    state = states.get(event["experiment"])
    if state is None:
        raise RegistryError(
            f"experiment {event['experiment']} is not open")
    if state["closed"] is not None:
        raise RegistryError(
            f"experiment {event['experiment']} is already closed")
    if not state["results"]:
        raise RegistryError(
            f"experiment {event['experiment']} has no result "
            f"to rest a verdict on")
    result_position, result_event = state["results"][-1]
    suggestion, regressed, unmeasured = suggested_verdict(
        state["opened"], result_event)
    event = {**event,
             "rests_on": {"baseline": state["opened_at_position"],
                          "result": result_position},
             "suggested": suggestion,
             "regressed": regressed,
             "unmeasured": unmeasured}
    defects = event_defects(event, _next_position(path))
    if defects:
        raise RegistryError("; ".join(defects))
    _append(path, event)
    return event


def list_view(path: Path) -> dict:
    events = _require_sound(path)
    states = fold(events)
    entries = []
    for state in states.values():
        entry = {"experiment": state["experiment"],
                 "source": state["opened"]["source"],
                 "proposal": state["opened"]["proposal"],
                 "state": state["state"],
                 "results": len(state["results"])}
        if state["closed"] is not None:
            entry["verdict"] = state["closed"]["verdict"]
            entry["suggested"] = state["closed"]["suggested"]
            entry["agrees"] = (
                state["closed"]["verdict"]
                == state["closed"]["suggested"])
        entries.append(entry)
    return {"experiments": entries}


def show_view(path: Path, experiment: str) -> dict:
    events = _require_sound(path)
    states = fold(events)
    state = states.get(experiment)
    if state is None:
        raise RegistryError(f"no experiment {experiment}")
    return {
        "experiment": experiment,
        "state": state["state"],
        "opened": {"position": state["opened_at_position"],
                   **state["opened"]},
        "results": [{"position": position, **event}
                    for position, event in state["results"]],
        "closed": ({"position": state["closed_at_position"],
                    **state["closed"]}
                   if state["closed"] is not None else None),
    }
