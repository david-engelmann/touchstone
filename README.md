# touchstone

An experiment registry for a fleet's setup. Every proposed change becomes a tracked experiment with a hypothesis, a run, measurements, and a verdict, keep or kill.

Touchstone is the lab notebook for a fleet's setup changes. A proposed change is opened as an experiment with its source, a baseline statement, baseline measures, and the measures it will be judged on, each with a direction and a margin. Results accumulate against the opening and the latest is current. Closing names the baseline and the current result by their positions in the file and records the closer's verdict beside a suggested verdict computed by a stated rule, and the suggestion is re-derived on every read, so a record whose suggestion does not recompute names itself as a defect. Touchstone judges changes, tameshi judges checks, and a tameshi ruling registers here as an experiment.

[![ci](https://github.com/david-engelmann/touchstone/actions/workflows/ci.yml/badge.svg)](https://github.com/david-engelmann/touchstone/actions/workflows/ci.yml)

```json
{
  "experiments": [
    {
      "experiment": "exp-00beff503e75f106",
      "source": "retroforge",
      "proposal": "rules-edit-1",
      "state": "closed",
      "results": 1,
      "verdict": "keep",
      "suggested": "keep",
      "agrees": true,
      "judgment": {
        "status": "not_consulted",
        "reason": "the verdict and the suggestion agree"
      }
    }
  ]
}
```

## Install

A Rust toolchain and Git are the prerequisites. Clone the repo and install the command from it.

```console
$ git clone https://github.com/david-engelmann/touchstone
$ cd touchstone
$ cargo install --path .
```

The installed command lands in Cargo's bin directory, $HOME/.cargo/bin by default, and the quickstart below assumes that directory is on the PATH. The crate builds on the flighttape crate at a pinned revision, and Cargo fetches that dependency during the install.

## Quickstart

Open an experiment, record a result, and close it with a verdict. The first three commands print nothing on success, and reading the experiment back gives the full record.

```console
$ touchstone open registry.jsonl exp-00beff503e75f106 --source retroforge --proposal rules-edit-1 --baseline "the week before" --measures '{"cost": 100, "rescue": 80}' --judged '[{"name": "cost", "direction": "lower"}, {"name": "rescue", "direction": "higher"}]' --at 2026-01-05T09:00:00Z
$ touchstone result registry.jsonl exp-00beff503e75f106 --measures '{"cost": 90, "rescue": 85}' --at 2026-01-12T09:00:00Z
$ touchstone close registry.jsonl exp-00beff503e75f106 --verdict keep --rationale "cost down and rescues up" --at 2026-01-12T10:00:00Z
$ touchstone show registry.jsonl exp-00beff503e75f106
{
  "experiment": "exp-00beff503e75f106",
  "state": "closed",
  "opened": {
    "position": 1,
    "type": "opened",
    "experiment": "exp-00beff503e75f106",
    "at": "2026-01-05T09:00:00Z",
    "source": "retroforge",
    "proposal": "rules-edit-1",
    "baseline": "the week before",
    "baseline_measures": {
      "cost": 100,
      "rescue": 80
    },
    "judged": [
      {
        "name": "cost",
        "direction": "lower"
      },
      {
        "name": "rescue",
        "direction": "higher"
      }
    ]
  },
  "results": [
    {
      "position": 2,
      "type": "result",
      "experiment": "exp-00beff503e75f106",
      "at": "2026-01-12T09:00:00Z",
      "measures": {
        "cost": 90,
        "rescue": 85
      }
    }
  ],
  "closed": {
    "position": 3,
    "type": "closed",
    "experiment": "exp-00beff503e75f106",
    "at": "2026-01-12T10:00:00Z",
    "verdict": "keep",
    "rationale": "cost down and rescues up",
    "rests_on": {
      "baseline": 1,
      "result": 2
    },
    "suggested": "keep",
    "regressed": [],
    "unmeasured": [],
    "judgment": {
      "status": "not_consulted",
      "reason": "the verdict and the suggestion agree"
    }
  }
}
```

## Usage

The registry is one append-only file of events, and its path is the first argument to every command. State is folded from the file on every read and nothing is stored beside it. The six commands are open, result, close, list, show, and check, and their full flags stand in each command's own help text.

The list command prints every experiment with its state, its verdict where one stands, and the suggestion beside it, which is the block at the top of this README. The check command re-reads the whole file, recomputes every experiment id and every recorded suggestion, and names every defect it finds.

```console
$ touchstone check registry.jsonl
the registry is sound
```

Closing rests on the baseline and the current result, named by their one-based positions in the file. Where the closer's verdict and the computed suggestion agree, no judge is consulted, as the quickstart record shows. Where they disagree, the close stands in the contested band and consults a judge named with the --judge flag, and a consulted judge's answer is recorded with the close when the close lands. When the judge abstains or answers below the 0.6 confidence floor, the close does not land, the experiment stays open, and nothing is recorded. A close can also name a successor experiment with the --successor flag.

An experiment that does not exist is refused by name on stderr, and the command exits 1.

```console
$ touchstone show registry.jsonl exp-0000000000000000
error, no experiment exp-0000000000000000
```

## Status

The registry and its six commands are complete at their contracted scope and run as a Rust binary, and judgment in the contested band is landed and recorded the same way as every other part of the record. The Python of the first round is retired and stands as the behavioral spec the port was verified against.

Closed perfection rounds, 6 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed.

## License

MIT. See the [LICENSE](LICENSE) file.
