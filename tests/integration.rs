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
        .args(&["add", "3", "tag", "another tag", "-c", "comment"]);
    cmd.assert().success();

    journal.assert(predicate::str::contains("| 3 | another tag,tag | comment"));
}

#[test]
fn check_mood() {
    let journal = prepare_empty_journal_file();

    let date_format = "%Y-%m-%d %H:%M:%S %z";
    let line_old = format!("{} | 4 | tag |\n", (Utc::now() - Duration::days(40)).format(date_format));
    let line_recent = format!("{} | 2 | tag2 |\n", Utc::now().format(date_format));
    let line_new = format!("{} | 1 | tag | foo", Utc::now().format(date_format));

    journal
        .write_str(
            format!("{}{}{}", line_old, line_recent, line_new).as_str()
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("howdy").unwrap();
    cmd.arg("-f")
        .arg(journal.path())
        .args(&["mood", "-t", "m"])
        .assert()
        .stdout("30-days mood: 3\n");

    let mut tagged_cmd = Command::cargo_bin("howdy").unwrap();
    tagged_cmd.arg("-f")
        .arg(journal.path())
        .args(&["mood", "tag", "--type", "m"])
        .assert()
        .stdout("30-days mood: 1\n");
}

fn prepare_empty_journal_file() -> assert_fs::NamedTempFile {
    let journal = assert_fs::NamedTempFile::new("howdy.journal").unwrap();
    journal.touch().unwrap();
    journal
}
