pub mod coreutils;
pub mod textutils;
pub mod fileutils;
pub mod misc;

use proteus_core::ProteusResult;

pub trait Applet {
    fn run(args: &[String]) -> ProteusResult<i32>;
}
