use crate::{
    parsing::{ExitedWithCode, StopReason},
    Gdb,
};

#[test]
fn gdb_spawns_and_dies() {
    let _gdb = Gdb::new().unwrap();
}

#[test]
fn gdb_runs_cat_until_completion() {
    let mut gdb = Gdb::new().unwrap();

    gdb.send_command("file cat").unwrap();

    let (_stop_message, stdout) = gdb.run("aaaa").unwrap();

    assert_eq!(String::from_utf8(stdout).unwrap(), "aaaa")
}

#[test]
fn gdb_detects_normal_exit() {
    let mut gdb = Gdb::new().unwrap();

    gdb.send_command("file vuln").unwrap();

    assert_eq!(gdb.run("aaa").unwrap().0, StopReason::ExitedNormally)
}

#[test]
fn gdb_detects_crash_signal() {
    let mut gdb = Gdb::new().unwrap();

    gdb.send_command("file vuln").unwrap();

    assert!(matches!(
        gdb.run(
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        )
        .unwrap()
        .0,
        StopReason::SignalReceived(_)
    ));
}

#[test]
fn gdb_detects_nonzero_exit() {
    let mut gdb = Gdb::new().unwrap();

    gdb.send_command("file vuln").unwrap();

    assert_eq!(
        gdb.run("AAAAAAB").unwrap().0,
        StopReason::Exited(ExitedWithCode { exit_code: 10 })
    )
}
