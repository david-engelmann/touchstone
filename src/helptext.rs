//! The help and usage texts, captured from the
//! Python's argparse at the default width and
//! pinned here; the parser prints them verbatim.

pub const TOP_HELP: &str = "usage: verdictloop [-h] {open,result,close,list,show,check} ...\n\npositional arguments:\n  {open,result,close,list,show,check}\n    open                open an experiment\n    result              record a result\n    close               close with a verdict\n    list                every experiment\n    show                one experiment\n    check               name every defect\n\noptions:\n  -h, --help            show this help message and exit\n";
pub const TOP_USAGE: &str = "usage: verdictloop [-h] {open,result,close,list,show,check} ...";

pub const OPEN_HELP: &str = "usage: verdictloop open [-h] --source SOURCE --proposal PROPOSAL --baseline\n                        BASELINE --measures MEASURES --judged JUDGED\n                        [--margin MARGIN] --at AT\n                        registry experiment\n\npositional arguments:\n  registry\n  experiment\n\noptions:\n  -h, --help           show this help message and exit\n  --source SOURCE\n  --proposal PROPOSAL\n  --baseline BASELINE\n  --measures MEASURES\n  --judged JUDGED\n  --margin MARGIN\n  --at AT\n";
pub const OPEN_USAGE: &str = "usage: verdictloop open [-h] --source SOURCE --proposal PROPOSAL --baseline\n                        BASELINE --measures MEASURES --judged JUDGED\n                        [--margin MARGIN] --at AT\n                        registry experiment";

pub const RESULT_HELP: &str = "usage: verdictloop result [-h] --measures MEASURES --at AT registry experiment\n\npositional arguments:\n  registry\n  experiment\n\noptions:\n  -h, --help           show this help message and exit\n  --measures MEASURES\n  --at AT\n";
pub const RESULT_USAGE: &str =
    "usage: verdictloop result [-h] --measures MEASURES --at AT registry experiment";

pub const CLOSE_HELP: &str = "usage: verdictloop close [-h] --verdict VERDICT --rationale RATIONALE --at AT\n                         [--successor SUCCESSOR] [--judge JUDGE]\n                         registry experiment\n\npositional arguments:\n  registry\n  experiment\n\noptions:\n  -h, --help            show this help message and exit\n  --verdict VERDICT\n  --rationale RATIONALE\n  --at AT\n  --successor SUCCESSOR\n  --judge JUDGE\n";
pub const CLOSE_USAGE: &str = "usage: verdictloop close [-h] --verdict VERDICT --rationale RATIONALE --at AT\n                         [--successor SUCCESSOR] [--judge JUDGE]\n                         registry experiment";

pub const LIST_HELP: &str = "usage: verdictloop list [-h] [-o OUTPUT] registry\n\npositional arguments:\n  registry\n\noptions:\n  -h, --help            show this help message and exit\n  -o OUTPUT, --output OUTPUT\n";
pub const LIST_USAGE: &str = "usage: verdictloop list [-h] [-o OUTPUT] registry";

pub const SHOW_HELP: &str = "usage: verdictloop show [-h] [-o OUTPUT] registry experiment\n\npositional arguments:\n  registry\n  experiment\n\noptions:\n  -h, --help            show this help message and exit\n  -o OUTPUT, --output OUTPUT\n";
pub const SHOW_USAGE: &str = "usage: verdictloop show [-h] [-o OUTPUT] registry experiment";

pub const CHECK_HELP: &str = "usage: verdictloop check [-h] registry\n\npositional arguments:\n  registry\n\noptions:\n  -h, --help  show this help message and exit\n";
pub const CHECK_USAGE: &str = "usage: verdictloop check [-h] registry";
