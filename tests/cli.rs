#![cfg(test)]

mod common;

mod cli {
    use predicates::prelude::*;
    use test_temp_dir::test_temp_dir;
    use trow::registry::{ImageValidationConfig, RegistryProxiesConfig, SingleRegistryProxyConfig};

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
            .args(["-n", "myhost.com"])
            .assert()
            .success()
            .stdout(predicate::str::contains(": \"myhost.com\""));

        get_command()
            .args(["--name", "trow.test"])
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

        get_command()
            .assert()
            .success()
            .stdout(predicate::str::contains("Proxy registries not configured"));

        let file = get_file(
            tmp_path,
            ImageValidationConfig {
                allow: vec!["trow.test/".to_string()],
                deny: vec!["toto".to_string()],
                default: "Allow".to_string(),
            },
        );

        get_command()
            .args(["--image-validation-config-file", file.to_str().unwrap()])
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
    fn registry_proxy() {
        let tmp_dir = test_temp_dir!();
        let tmp_path = tmp_dir.as_path_untracked();

        get_command()
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Image validation webhook not configured",
            ));

        let file = get_file(
            tmp_path,
            RegistryProxiesConfig {
                offline: true,
                registries: vec![
                    SingleRegistryProxyConfig {
                        alias: "lovni".to_string(),
                        host: "jul.example.com".to_string(),
                        username: Some("robert".to_string()),
                        password: Some("1234".to_string()),
                        ignore_repos: vec![],
                    },
                    SingleRegistryProxyConfig {
                        alias: "trow".to_string(),
                        host: "127.0.0.1".to_string(),
                        username: None,
                        password: None,
                        ignore_repos: vec![],
                    },
                ],
            },
        );

        get_command()
            .args(["--proxy-registry-config-file", file.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                [
                    "Proxy registries configured:",
                    "  - lovni: jul.example.com",
                    "  - trow: 127.0.0.1",
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
