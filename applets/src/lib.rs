pub mod coreutils;

use proteus_core::ProteusResult;

pub trait Applet {
    fn run(args: &[String]) -> ProteusResult<i32>;
}
