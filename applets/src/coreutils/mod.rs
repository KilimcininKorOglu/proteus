use proteus_core::ProteusResult;

#[cfg(feature = "cat")]
pub mod cat;
#[cfg(feature = "ls")]
pub mod ls;
#[cfg(feature = "cp")]
pub mod cp;
#[cfg(feature = "mv")]
pub mod mv;
#[cfg(feature = "rm")]
pub mod rm;
#[cfg(feature = "echo")]
pub mod echo;
#[cfg(feature = "head")]
pub mod head;
#[cfg(feature = "tail")]
pub mod tail;
#[cfg(feature = "wc")]
pub mod wc;
#[cfg(feature = "pwd")]
pub mod pwd;
#[cfg(feature = "mkdir")]
pub mod mkdir_applet;
#[cfg(feature = "rmdir")]
pub mod rmdir;
#[cfg(feature = "touch")]
pub mod touch;
#[cfg(feature = "chmod")]
pub mod chmod;
#[cfg(feature = "chown")]
pub mod chown;
#[cfg(feature = "chgrp")]
pub mod chgrp;
#[cfg(feature = "ln")]
pub mod ln;
#[cfg(feature = "basename")]
pub mod basename;
#[cfg(feature = "dirname")]
pub mod dirname;
#[cfg(feature = "printf")]
pub mod printf;
#[cfg(feature = "tee")]
pub mod tee;
#[cfg(feature = "env")]
pub mod env;
#[cfg(feature = "uname")]
pub mod uname;
#[cfg(feature = "id")]
pub mod id;
#[cfg(feature = "whoami")]
pub mod whoami;
#[cfg(feature = "groups")]
pub mod groups;
#[cfg(feature = "true")]
pub mod true_cmd;
#[cfg(feature = "false")]
pub mod false_cmd;

#[cfg(feature = "cat")]
pub fn run_cat(args: &[String]) -> ProteusResult<i32> {
    cat::run(args)
}

#[cfg(feature = "ls")]
pub fn run_ls(args: &[String]) -> ProteusResult<i32> {
    ls::run(args)
}

#[cfg(feature = "cp")]
pub fn run_cp(args: &[String]) -> ProteusResult<i32> {
    cp::run(args)
}

#[cfg(feature = "mv")]
pub fn run_mv(args: &[String]) -> ProteusResult<i32> {
    mv::run(args)
}

#[cfg(feature = "rm")]
pub fn run_rm(args: &[String]) -> ProteusResult<i32> {
    rm::run(args)
}

#[cfg(feature = "echo")]
pub fn run_echo(args: &[String]) -> ProteusResult<i32> {
    echo::run(args)
}

#[cfg(feature = "head")]
pub fn run_head(args: &[String]) -> ProteusResult<i32> {
    head::run(args)
}

#[cfg(feature = "tail")]
pub fn run_tail(args: &[String]) -> ProteusResult<i32> {
    tail::run(args)
}

#[cfg(feature = "wc")]
pub fn run_wc(args: &[String]) -> ProteusResult<i32> {
    wc::run(args)
}

#[cfg(feature = "pwd")]
pub fn run_pwd(args: &[String]) -> ProteusResult<i32> {
    pwd::run(args)
}

#[cfg(feature = "mkdir")]
pub fn run_mkdir(args: &[String]) -> ProteusResult<i32> {
    mkdir_applet::run(args)
}

#[cfg(feature = "rmdir")]
pub fn run_rmdir(args: &[String]) -> ProteusResult<i32> {
    rmdir::run(args)
}

#[cfg(feature = "touch")]
pub fn run_touch(args: &[String]) -> ProteusResult<i32> {
    touch::run(args)
}

#[cfg(feature = "chmod")]
pub fn run_chmod(args: &[String]) -> ProteusResult<i32> {
    chmod::run(args)
}

#[cfg(feature = "chown")]
pub fn run_chown(args: &[String]) -> ProteusResult<i32> {
    chown::run(args)
}

#[cfg(feature = "chgrp")]
pub fn run_chgrp(args: &[String]) -> ProteusResult<i32> {
    chgrp::run(args)
}

#[cfg(feature = "ln")]
pub fn run_ln(args: &[String]) -> ProteusResult<i32> {
    ln::run(args)
}

#[cfg(feature = "basename")]
pub fn run_basename(args: &[String]) -> ProteusResult<i32> {
    basename::run(args)
}

#[cfg(feature = "dirname")]
pub fn run_dirname(args: &[String]) -> ProteusResult<i32> {
    dirname::run(args)
}

#[cfg(feature = "printf")]
pub fn run_printf(args: &[String]) -> ProteusResult<i32> {
    printf::run(args)
}

#[cfg(feature = "tee")]
pub fn run_tee(args: &[String]) -> ProteusResult<i32> {
    tee::run(args)
}

#[cfg(feature = "env")]
pub fn run_env(args: &[String]) -> ProteusResult<i32> {
    env::run(args)
}

#[cfg(feature = "uname")]
pub fn run_uname(args: &[String]) -> ProteusResult<i32> {
    uname::run(args)
}

#[cfg(feature = "id")]
pub fn run_id(args: &[String]) -> ProteusResult<i32> {
    id::run(args)
}

#[cfg(feature = "whoami")]
pub fn run_whoami(args: &[String]) -> ProteusResult<i32> {
    whoami::run(args)
}

#[cfg(feature = "groups")]
pub fn run_groups(args: &[String]) -> ProteusResult<i32> {
    groups::run(args)
}

#[cfg(feature = "true")]
pub fn run_true(args: &[String]) -> ProteusResult<i32> {
    true_cmd::run(args)
}

#[cfg(feature = "false")]
pub fn run_false(args: &[String]) -> ProteusResult<i32> {
    false_cmd::run(args)
}
