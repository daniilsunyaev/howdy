use assert_fs::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use chrono::{Duration, Utc};

#[test]
fn add_record_to_journal() {
    let journal = prepare_empty_journal_file();

    let mut cmd = Command::cargo_bin("howdy").unwrap();
    cmd.arg("-f")
        .arg(journal.path())
        .args(&["add", "3", "comment"]);
    cmd.assert().success();

    journal.assert(predicate::str::contains("| 3 | comment"));
}

#[test]
fn check_mood() {
    let journal = prepare_empty_journal_file();

    let date_format = "%Y-%m-%d %H:%M:%S %z";
    let line_old = format!("{} | {} | \n", (Utc::now() - Duration::days(40)).format(date_format), 4);
    let line_recent = format!("{} | {} | \n", Utc::now().format(date_format), 2);
    let line_new = format!("{} | {} | foo", Utc::now().format(date_format), 1);

    journal
        .write_str(
            format!("{}{}{}", line_old, line_recent, line_new).as_str()
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("howdy").unwrap();
    cmd.arg("-f")
        .arg(journal.path())
        .args(&["mood", "m"])
        .assert()
        .stdout("30-days mood: 3\n");
}

fn prepare_empty_journal_file() -> assert_fs::NamedTempFile {
    let journal = assert_fs::NamedTempFile::new("howdy.journal").unwrap();
    journal.touch().unwrap();
    journal
}
