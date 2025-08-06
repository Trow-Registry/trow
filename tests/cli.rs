#![cfg(test)]

mod common;

mod cli {
    use predicates::prelude::*;
    use test_temp_dir::test_temp_dir;
    use trow::registry::{ConfigFile, ImageValidationConfig};

    use crate::common::get_file;

    fn get_command() -> assert_cmd::Command {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg("--dry-run");
        cmd
    }

    #[test]
    fn invalid_argument() {
        get_command()
            .arg("-Z")
            .assert()
            .stderr(predicate::str::contains(
                "error: unexpected argument '-Z' found",
            ))
            .failure();
    }

    #[test]
    fn help_works() {
        get_command()
            .arg("-h")
            .assert()
            .success()
            .stdout(predicate::str::contains("Trow"));

        get_command()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Trow"));
    }

    #[test]
    fn host_name_parsing() {
        get_command()
            .args(["--hostname", "myhost.com"])
            .assert()
            .success()
            .stdout(predicate::str::contains(": \"myhost.com\""));

        get_command()
            .args(["--hostname", "trow.test"])
            .assert()
            .success()
            .stdout(predicate::str::contains(": \"trow.test\""));

        get_command()
            .args(["-n=port.io:3833"])
            .assert()
            .success()
            .stdout(predicate::str::contains(": \"port.io:3833\""));
    }

    #[test]
    fn image_validation() {
        let tmp_dir = test_temp_dir!();
        let tmp_path = tmp_dir.as_path_untracked();

        let file = get_file(
            tmp_path,
            ConfigFile {
                image_validation: Some(ImageValidationConfig {
                    allow: vec!["trow.test/".to_string()],
                    deny: vec!["toto".to_string()],
                    default: "Allow".to_string(),
                }),
                ..Default::default()
            },
        );

        get_command()
            .args(["--config-file", file.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                [
                    "Image validation webhook configured:",
                    "  Default action: Allow",
                    "  Allowed prefixes: [\"trow.test/\"]",
                    "  Denied prefixes: [\"toto\"]",
                ]
                .join("\n"),
            ));
    }

    #[test]
    fn cors() {
        get_command()
            .args(["--cors", "ftp://trow.test"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Cross-Origin Resource Sharing(CORS) requests are allowed",
            ));
    }
}
