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

}
