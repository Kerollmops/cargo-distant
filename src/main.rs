#[macro_use] extern crate structopt;

use std::{env, process};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::path::Path;
use std::ffi::OsStr;
use structopt::StructOpt;

// Don't forget to update the Command enum
const SUBCOMMANDS: &[&str] = &["build", "check", "doc", "run", "test", "bench"];

// Don't forget to update the SUBCOMMANDS const
#[derive(StructOpt, Debug)]
enum Command {

    /// Build on the remote machine and copy-back on the local machine
    #[structopt(name = "build")]
    Build,

    /// Check the code on the remote machine and copy-back on the local machine
    #[structopt(name = "check")]
    Check,

    /// Doc on the remote machine then copy-back on the local machine
    #[structopt(name = "doc")]
    Doc,

    /// Build on the remote machine then copy-back and `run` on the local machine
    #[structopt(name = "run")]
    Run,

    /// Build on the remote machine with the `--tests` option then copy-back and `test` on the local machine
    #[structopt(name = "test")]
    Test,

    /// Build on the remote machine with the `--benches` option then copy-back and `bench` on the local machine
    #[structopt(name = "bench")]
    Bench,
}

/// Every command copy-back by default and display their output on the standard output.
#[derive(StructOpt, Debug)]
#[structopt(name = "cargo-distant", bin_name = "cargo distant")]
struct Opts {
    #[structopt(help = "Can be defined in the `distant.toml` file")]
    hostname: Option<String>,

    #[structopt(long = "no-copy-back")]
    no_copy_back: bool,

    // TODO retrieve actual toolchain from local-machine
    #[structopt(long = "toolchain", default_value = "stable")]
    toolchain: String,

    #[structopt(subcommand)]
    command: Command,
}

impl Opts {
    fn execute<P, I, S>(&self, path: P, args: I) -> Result<(), ()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
        P: AsRef<Path>,
    {
        let ssh_cmd = match env::var_os("DISTANT_SSH_COMMAND") {
            Some(var) => var,
            None => {
                let hostname = match self.hostname.as_ref() {
                    Some(name) => name,
                    None => unimplemented!("search in toml file"),
                };
                format!("ssh {}", hostname).into()
            },
        };

        let cargo_cmd = match env::var_os("DISTANT_CARGO_COMMAND") {
            Some(var) => var,
            None => format!("$HOME/.cargo/run/rustup run {}", self.toolchain).into(),
        };

        let project_path = {
            let mut hasher = DefaultHasher::default();

            let path = path.as_ref();

            // TODO make and explain that the remote-path is
            //      hash($hostname + $USER + $local-path)
            path.hash(&mut hasher);

            let post_name = path.file_name()
                              .map(|x| x.to_string_lossy())
                              .unwrap_or("xxx".into());

            format!("$HOME/.distant/{}-{:#x}", post_name, hasher.finish())
        };

        let mut sh_c = process::Command::new("sh");
        let base = sh_c.arg("-c")
                       .arg(ssh_cmd).arg("--") // from here, arg(s) are useless
                       .args(&["set", "-e"])
                       .args(&["cd", &project_path, ";"])
                       .arg(cargo_cmd)
                       .stdin(process::Stdio::null());

        match self.command {
            Command::Build => {
                let status = base.arg("build")
                                .args(args)
                                .status();

                println!("{:?}", status);
            },
            Command::Check => {
                //
            },
            Command::Doc => {
                //
            },
            Command::Run => {
                //
            },
            Command::Test => {
                //
            },
            Command::Bench => {
                //
            },
        }

        Ok(())
    }
}

fn main() {
    let mut args: Vec<_> = env::args().collect();

    if let Some(build_pos) = args.iter().position(|s| SUBCOMMANDS.contains(&s.as_str())) {
        let build_args = args.split_off(build_pos + 1);
        let before_build = args;

        let matches = Opts::from_any(&before_build[1..]);

        // TODO why do we need the local path "." ?
        let pwd = env::current_dir().unwrap(); // FIXME
        matches.execute(pwd, build_args);

    } else {
        let _ = Opts::clap().print_help();
        // exit 1
    }
}
