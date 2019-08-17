//! Compare and sync maintainers from Nixpkgs to maintainers on
//! GitHub Maintainer team, as described in RFC #39:
//! https://github.com/NixOS/rfcs/blob/master/rfcs/0039-unprivileged-maintainer-teams.md

#![warn(missing_docs)]

#[macro_use]
extern crate slog;

#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;
use structopt::StructOpt;
mod maintainers;
use maintainers::MaintainerList;
mod filemunge;
mod maintainerhistory;
mod nix;
mod op_backfill;
mod op_blame_author;
mod op_check_handles;
use hubcaps::{Credentials, Github};
use std::env;

#[derive(Debug, StructOpt)]
struct Options {
    /// Maintainer list
    #[structopt(short = "m", long = "maintainers", parse(from_os_str))]
    maintainers: PathBuf,

    /// Execution Mode
    #[structopt(subcommand)]
    mode: ExecMode,
}

#[derive(Debug, StructOpt)]
enum ExecMode {
    /// Verify maintainers, their GitHub handle, and GitHub ID
    #[structopt(name = "check-handles")]
    CheckHandles,

    /// Poorly edit the maintainers.nix file to add missing GitHub IDs
    #[structopt(name = "backfill-ids")]
    BackfillIDs,

    /// Look to see if any of the GitHub handles have probably changed
    /// by examining who authored the commit adding the maintainer
    /// to the .nix file.
    #[structopt(name = "blame-author")]
    BlameAuthor,
}

fn main() {
    let logger = rfc39::default_logger();

    let inputs = Options::from_args();

    let maintainers_file = inputs.maintainers.canonicalize().unwrap();
    info!(logger, "Loading maintainer information";
          "from" => inputs.maintainers.display(),
          "absolute" => maintainers_file.display()
    );

    let maintainers = MaintainerList::load(logger.clone(), &maintainers_file).unwrap();

    let github = Github::new(
        String::from("NixOS/rfcs#39 (hubcaps)"),
        env::var("GITHUB_TOKEN").ok().map(Credentials::Token),
    )
    .unwrap();

    match inputs.mode {
        ExecMode::CheckHandles => op_check_handles::check_handles(
            logger.new(o!("exec-mode" => "CheckHandles")),
            maintainers,
        ),
        ExecMode::BackfillIDs => op_backfill::backfill_ids(
            logger.new(o!("exec-mode" => "BackfillIDs")),
            github,
            &maintainers_file,
            maintainers,
        ),
        ExecMode::BlameAuthor => op_blame_author::report(
            logger.new(o!("exec-mode" => "BlameAuthor")),
            github,
            &maintainers_file,
            maintainers,
        ),
    }
}
