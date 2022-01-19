#[cfg(test)]
mod cli {
    use predicates::prelude::*;

    fn get_command() -> assert_cmd::Command {
        assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
    }

    #[test]
    fn invalid_argument() {
        get_command()
            .arg("-Z")
            .assert()
            .stderr(predicate::str::contains("Found argument '-Z' which wasn't expected, or isn't valid in this context"))
            .failure();

        get_command()
            .arg("-Z")
            .assert()
            .failure()
            .stderr(predicate::str::contains("error: Found argument '-Z' which wasn't expected"));
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
            .args(&["-n", "myhost.com", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[\"myhost.com\"]"));

        get_command()
            .args(&["--names", "trow.test", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[\"trow.test\"]"));

        get_command()
            .args(&["-n myhost.com second", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[\"myhost.com\", \"second\"]"));

        get_command()
            .args(&["-n port.io:3833 second", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("[\"port.io:3833\", \"second\"]"));
    }

    #[test]
    fn image_validation() {
        get_command()
            .args(&[
                "--deny-k8s-images",
                "--allow-prefixes",
                "myreg.com/",
                "--dry-run",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Images with these prefixes are explicitly allowed: [\"myreg.com/\"]"));


        get_command()
            .args(&["--allow-images", "myreg.com/myimage:1.2", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Images with these names are explicitly allowed: [\"myreg.com/myimage:1.2\"]"));


        get_command()
            .args(&["--disallow-local-images", "myimage:1.2", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Local images with these names are explicitly denied: [\"myimage:1.2\"]"));


        get_command()
            .args(&["--disallow-local-prefixes", "beta/", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Local images with these prefixes are explicitly denied: [\"beta/\"]"));
    }

    #[test]
    fn cors() {
        get_command()
            .args(&["--enable-cors", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Cross-Origin Resource Sharing(CORS) requests are allowed"));
    }

    #[test]
    fn file_size_parsing() {
        get_command()
            .args(&["--max-manifest-size", "3", "--dry-run"])
            .assert()
            .success()
            .stdout(predicate::str::contains("manifest size: 3"));


        get_command()
            .args(&["--max-manifest-size", "-4"])
            .assert()
            .failure();

        get_command()
            .args(&["--max-manifest-size", "1.1"])
            .assert()
            .failure();
    }

    #[test]
    fn log_level_setting() {
        get_command()
            .args(&["--log-level", "TRACE", "--dry-run"])
            .assert()
            .success();
    }
}
