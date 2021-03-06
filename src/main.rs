#![cfg_attr(feature="lints", feature(plugin))]
#![cfg_attr(feature="lints", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate toml;
extern crate semver;
extern crate tempdir;
#[cfg(feature = "color")]
extern crate ansi_term;
extern crate tabwriter;

#[macro_use]
mod macros;
mod config;
mod lockfile;
mod deps;
mod error;
mod fmt;

use std::io::{Write, stdout};
#[cfg(feature="debug")]
use std::env;

use clap::{App, Arg, SubCommand, AppSettings};
use tabwriter::TabWriter;

use config::Config;
use lockfile::Lockfile;
use error::CliError;
use fmt::Format;

pub type CliResult<T> = Result<T, CliError>;

fn main() {
    debugln!("executing; cmd=cargo-outdated; args={:?}", env::args().collect::<Vec<_>>());
    let m = App::new("cargo-outdated")
        .author("Kevin K. <kbknapp@gmail.com>")
        .about("Displays information about project dependency versions")
        .version(&*format!("v{}", crate_version!()))
        // We have to lie about our binary name since this will be a third party
        // subcommand for cargo
        .bin_name("cargo")
        // Global version uses the version we supplied (Cargo.toml) for all subcommands as well
        .settings(&[AppSettings::GlobalVersion,
                    AppSettings::SubcommandRequired])
        // We use a subcommand because parsed after `cargo` is sent to the third party plugin
        // which will be interpreted as a subcommand/positional arg by clap
        .subcommand(SubCommand::with_name("outdated")
            .about("Displays information about project dependency versions")
            .args_from_usage("-p, --package [PKG]...    'Package to inspect for updates'
                              -v, --verbose             'Print verbose output'
                              -d, --depth [DEPTH]       'How deep in the dependency chain to search{n}\
                                                         (Defaults to all dependencies when omitted)'")
            // We separate -R so we can addd a conflicting argument
            .arg(Arg::from_usage("-R, --root-deps-only  'Only check root dependencies (Equivilant to --depth=1)'")
                .conflicts_with("DEPTH")))

        .get_matches();

    if let Some(m) = m.subcommand_matches("outdated") {
        let cfg = Config::from_matches(m);
        if let Err(e) = execute(cfg) {
            e.exit();
        }
    }
}

fn execute(cfg: Config) -> CliResult<()> {
    debugln!("executing; execute; cfg={:?}", cfg);

    verbose!(cfg, "Parsing {}...", Format::Warning("Cargo.lock"));
    let mut lf = try!(Lockfile::new());
    verboseln!(cfg, "{}", Format::Good("Done"));

    if let Ok(Some(res)) = lf.get_updates(&cfg) {
        println!("The following dependencies have newer versions available:\n");
        let mut tw = TabWriter::new(vec![]);
        write!(&mut tw, "\tName\tProject Ver\tSemVer Compat\tLatest Ver\n").unwrap();
        for (d_name, d) in res.iter() {
            write!(&mut tw, "\t{}\t   {}\t   {}\t  {}\n", d_name, d.project_ver, d.semver_ver.as_ref().unwrap_or(&String::from("  --  ")), d.latest_ver.as_ref().unwrap_or(&String::from("  --  "))).unwrap();
        }
        tw.flush().unwrap();
        write!(stdout(), "{}", String::from_utf8(tw.unwrap()).unwrap()).unwrap();
    } else {
        println!("All dependencies are up to date, yay!")
    }

    Ok(())
}


