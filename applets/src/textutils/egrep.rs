use proteus_core::ProteusResult;

use super::{run_grep, MatchMode};
use proteus_core::regex::RegexSyntax;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    run_grep(args, MatchMode::Regex(RegexSyntax::Extended), "egrep")
}
