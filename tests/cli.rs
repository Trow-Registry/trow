extern crate assert_cli;

#[cfg(test)]
mod cli {
    use assert_cli;

    #[test]
    fn invalid_argument() {
        assert_cli::Assert::main_binary()
            .with_args(&["-Z"])
            .fails()
            .and()
            .stderr()
            .contains("error: Found argument '-Z' which wasn't expected")
            .unwrap();
    }

    #[test]
    fn help_works() {
        assert_cli::Assert::main_binary()
            .with_args(&["-h"])
            .succeeds()
            .and()
            .stdout()
            .contains("Trow")
            .unwrap();

        assert_cli::Assert::main_binary()
            .with_args(&["--help"])
            .succeeds()
            .and()
            .stdout()
            .contains("Trow")
            .unwrap();
    }

    #[test]
    fn host_name_parsing() {
        assert_cli::Assert::main_binary()
            .with_args(&["-n myhost.com", "--dry-run"])
            .succeeds()
            .and()
            .stdout()
            .contains("[\"myhost.com\"]")
            .unwrap();

        assert_cli::Assert::main_binary()
            .with_args(&["--names", "trow.test", "--dry-run"])
            .succeeds()
            .and()
            .stdout()
            .contains("[\"trow.test\"]")
            .unwrap();

        assert_cli::Assert::main_binary()
            .with_args(&["-n myhost.com second", "--dry-run"])
            .succeeds()
            .and()
            .stdout()
            .contains("[\"myhost.com\", \"second\"]")
            .unwrap();

        assert_cli::Assert::main_binary()
            .with_args(&["-n port.io:3833 second", "--dry-run"])
            .succeeds()
            .and()
            .stdout()
            .contains("[\"port.io:3833\", \"second\"]")
            .unwrap();
    }

    #[test]
    fn image_validation() {
        assert_cli::Assert::main_binary()
            .with_args(&[
                "--deny-k8s-images",
                "--allow-prefixes",
                "myreg.com/",
                "--dry-run",
            ])
            .succeeds()
            .and()
            .stdout()
            .contains("Images with these prefixes are explicitly allowed: [\"myreg.com/\"]")
            .unwrap();

        assert_cli::Assert::main_binary()
            .with_args(&["--allow-images", "myreg.com/myimage:1.2", "--dry-run"])
            .succeeds()
            .and()
            .stdout()
            .contains("Images with these names are explicitly allowed: [\"myreg.com/myimage:1.2\"]")
            .unwrap();

        assert_cli::Assert::main_binary()
            .with_args(&["--disallow-local-images", "myimage:1.2", "--dry-run"])
            .succeeds()
            .and()
            .stdout()
            .contains("Local images with these names are explicitly denied: [\"myimage:1.2\"]")
            .unwrap();

        assert_cli::Assert::main_binary()
            .with_args(&["--disallow-local-prefixes", "beta/", "--dry-run"])
            .succeeds()
            .and()
            .stdout()
            .contains("Local images with these prefixes are explicitly denied: [\"beta/\"]")
            .unwrap();
    }

}
