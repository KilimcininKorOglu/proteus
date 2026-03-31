use proteus_core::ProteusResult;

use super::{run_grep, MatchMode};

pub fn run(args: &[String]) -> ProteusResult<i32> {
    run_grep(args, MatchMode::Fixed, "fgrep")
}
