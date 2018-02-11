#[macro_use] extern crate structopt;
extern crate welder;

use std::{env, process};
use std::io::Write;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::path::Path;
use structopt::StructOpt;
use welder::Welder;

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
    fn execute<P, I>(&self, path: P, args: I) -> Result<(), ()>
    where
        I: IntoIterator<Item = String>, // TODO String ?
        P: AsRef<Path>,
    {
        let ssh_cmd = ssh_command(&self.hostname);

        let rustup_run = match env::var("DISTANT_CARGO_COMMAND") {
            Ok(var) => var,
            Err(err) => format!("$HOME/.cargo/bin/rustup run {} cargo", self.toolchain).into(),
        };

        let project_path = project_path(path);

        let mut sh = process::Command::new("sh");
        let ssh_command = sh.arg("-c").arg(ssh_cmd).stdin(process::Stdio::piped());

        let command = Welder::new(' ')
                            .elems(vec!["set", "-e", "&&"])
                            // .elems(vec!["mkdir", "-p", &project_path, "&&"])
                            .elems(vec!["cd", &project_path, "&&"])
                            .elem(rustup_run);

        match self.command {
            Command::Build => {
                let command = command.elem("build").elems(args);

                match ssh_command.spawn() {
                    Ok(mut child) => {
                        {
                            let mut stdin = child.stdin.as_mut().expect("Failed to open stdin"); // FIXME wrong !
                            let command: String = command.weld();
                            stdin.write_all(command.as_bytes()).expect("Failed to write to stdin"); // FIXME wrong !
                        }

                        eprintln!("Trying to contact the remote machine...");
                        let status = child.wait();
                        println!("{:?}", status);
                    },
                    Err(err) => panic!("{:?}", err),
                }
            },
            Command::Check => {
                // command + "check"
                unimplemented!()
            },
            Command::Doc => {
                unimplemented!()
            },
            Command::Run => {
                unimplemented!()
            },
            Command::Test => {
                unimplemented!()
            },
            Command::Bench => {
                unimplemented!()
            },
        };

        Ok(())
    }
}

fn ssh_command(hostname: &Option<String>) -> String {
    // TODO does the env variable is the priority ? or the argument ?
    match *hostname {
        Some(ref host) => format!("ssh {}", host),
        None => {
            // TODO search in distant.toml config files

            match env::var("DISTANT_SSH_COMMAND") {
                Ok(var) => var,
                Err(err) => panic!("Whoops no hostname found"),
            }
        }
    }
}

fn project_path<P: AsRef<Path>>(path: P) -> String {
    let mut hasher = DefaultHasher::default();

    let path = path.as_ref();

    // TODO make and explain that the remote-path is
    //      hash($hostname + $USER + $local-path)
    path.hash(&mut hasher);

    let prefix_name = path.file_name()
                          .map(|x| x.to_string_lossy())
                          .unwrap_or("xxx".into());

    format!("$HOME/.distant/{}-{:x}", prefix_name, hasher.finish())
}

fn main() {
    let mut args: Vec<_> = env::args().collect();

    if let Some(build_pos) = args.iter().position(|s| SUBCOMMANDS.contains(&s.as_str())) {
        let build_args = args.split_off(build_pos + 1);
        let before_build = args;

        let matches = Opts::from_iter(&before_build[1..]);

        let pwd = env::current_dir().unwrap(); // FIXME
        matches.execute(pwd, build_args);

    } else {
        let _ = Opts::clap().print_help();
        // exit 1
    }
}
