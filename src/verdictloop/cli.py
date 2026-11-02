"""VL-12. A thin command line: open, result, close,
list, show, check. Failures are named errors with
exit 1."""
from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

from .registry import (RegistryError, add_result, close_experiment,
                       list_view, load_registry, open_experiment,
                       show_view)


def _parse_json(text: str, what: str):
    try:
        return json.loads(text)
    except ValueError:
        raise RegistryError(f"{what} is not valid JSON") from None


def _write_atomically(path: Path, text: str) -> None:
    fd, tmp = tempfile.mkstemp(dir=str(path.parent),
                               prefix=path.name + ".")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.replace(tmp, path)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _emit(document, out) -> int:
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    if out is not None:
        try:
            _write_atomically(Path(out), text)
        except (OSError, UnicodeEncodeError):
            print(f"error, cannot write the view, {out}",
                  file=sys.stderr)
            return 1
        return 0
    try:
        sys.stdout.write(text)
        sys.stdout.flush()
    except (OSError, UnicodeEncodeError):
        print("error, cannot write to stdout", file=sys.stderr)
        try:
            sys.stdout.close()
        except OSError:
            pass
        sys.stdout = open(os.devnull, "w", encoding="utf-8")
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="verdictloop")
    sub = parser.add_subparsers(dest="command", required=True)
    op = sub.add_parser("open", help="open an experiment")
    op.add_argument("registry")
    op.add_argument("experiment")
    op.add_argument("--source", required=True)
    op.add_argument("--proposal", required=True)
    op.add_argument("--baseline", required=True)
    op.add_argument("--measures", required=True)
    op.add_argument("--judged", required=True)
    op.add_argument("--margin", type=float)
    op.add_argument("--at", required=True)
    rs = sub.add_parser("result", help="record a result")
    rs.add_argument("registry")
    rs.add_argument("experiment")
    rs.add_argument("--measures", required=True)
    rs.add_argument("--at", required=True)
    cl = sub.add_parser("close", help="close with a verdict")
    cl.add_argument("registry")
    cl.add_argument("experiment")
    cl.add_argument("--verdict", required=True)
    cl.add_argument("--rationale", required=True)
    cl.add_argument("--at", required=True)
    cl.add_argument("--successor")
    li = sub.add_parser("list", help="every experiment")
    li.add_argument("registry")
    li.add_argument("-o", "--output")
    sh = sub.add_parser("show", help="one experiment")
    sh.add_argument("registry")
    sh.add_argument("experiment")
    sh.add_argument("-o", "--output")
    ck = sub.add_parser("check", help="name every defect")
    ck.add_argument("registry")
    args = parser.parse_args(argv)
    path = Path(args.registry)
    out = getattr(args, "output", None)
    if out is not None and Path(out).resolve() == path.resolve():
        print("error, a view never overwrites the registry",
              file=sys.stderr)
        return 1
    try:
        if args.command == "open":
            event = {"type": "opened",
                     "experiment": args.experiment,
                     "at": args.at, "source": args.source,
                     "proposal": args.proposal,
                     "baseline": args.baseline,
                     "baseline_measures": _parse_json(
                         args.measures, "the baseline measures"),
                     "judged": _parse_json(args.judged,
                                           "the judged list")}
            if args.margin is not None:
                event["margin_pct"] = args.margin
            open_experiment(path, event)
            return 0
        if args.command == "result":
            add_result(path, {
                "type": "result", "experiment": args.experiment,
                "at": args.at,
                "measures": _parse_json(args.measures,
                                        "the measures")})
            return 0
        if args.command == "close":
            event = {"type": "closed",
                     "experiment": args.experiment,
                     "at": args.at, "verdict": args.verdict,
                     "rationale": args.rationale}
            if args.successor:
                event["successor"] = args.successor
            close_experiment(path, event)
            return 0
        if args.command == "list":
            return _emit(list_view(path), args.output)
        if args.command == "show":
            return _emit(show_view(path, args.experiment),
                         args.output)
        if args.command == "check":
            _, defects = load_registry(path)
            for defect in defects:
                print(defect)
            if defects:
                return 1
            print("the registry is sound")
            return 0
    except RegistryError as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    except OSError as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
