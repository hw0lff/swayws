use std::convert::TryFrom;
use std::num::ParseIntError;

use clap::{Args, Parser, Subcommand};

/// SwayWs
/// allows easy moving of workspaces to and from outputs
#[derive(Debug, Parser)]
#[clap(name = "swayws", version)]
pub(crate) struct SwayWs {
    #[clap(subcommand)]
    pub(crate) cmd: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Focus a workspace
    #[clap(alias = "f")]
    Focus(Focus),
    /// Lists infos about sway
    #[clap(alias = "l")]
    List {
        // todo: add options to list first and last entry
        /// List outputs
        #[clap(short, long)]
        outputs: bool,

        /// List workspaces
        #[clap(short, long)]
        workspaces: bool,
    },
    /// Moves a workspace to a specified output
    #[clap(alias = "m")]
    Move {
        /// Moves workspace to output that does not match the specified output name
        #[clap(short, long)]
        away: bool,
        /// Excludes outputs to move workspace to,
        /// must be used with --away
        #[clap(long, requires("away"))]
        not: Option<Vec<String>>,
        /// Focuses specified workspace after moving it
        #[clap(short, long)]
        focus: bool,

        /// Workspace to move
        workspace: String,

        /// Name of the output
        output: String,
    },
    /// Moves a range of workspaces to a specified output
    #[clap(alias = "r")]
    Range {
        /// Moves workspace to output that does not match the specified output name
        #[clap(short, long)]
        away: bool,
        /// Excludes outputs to move workspace to,
        /// must be used with --away
        #[clap(long, requires("away"))]
        not: Option<Vec<String>>,

        /// Assumes <start> and <end> are numbers and binds all workspaces in between them to the specified output
        #[clap(short, long)]
        numeric: bool,

        /// First workspace in range
        start: String,
        /// Last workspace in range
        end: String,

        /// Name of the output
        output: String,
    },
    /// Swaps two workspaces with each other
    #[clap(alias = "s")]
    Swap {
        #[clap(value_name = "WORKSPACE", hide = true)]
        ws_l: String,
        #[clap(value_name = "WORKSPACE", hide = true)]
        ws_r: String,
    },
}

#[derive(Clone, Debug, Args)]
#[clap(group = clap::ArgGroup::new("focus_by").multiple(false))]
pub(crate) struct Focus {
    /// Focus the workspace by name.
    /// This is the default action if no flag is specified.
    #[arg(long, group = "focus_by")]
    name: bool,

    /// Focus the workspace by num.
    #[arg(long, group = "focus_by")]
    num: bool,

    /// Focus the workspace by id.
    #[arg(long, group = "focus_by")]
    id: bool,

    /// Try to focus the workspace a bit smartly.
    ///
    /// By trying to find the workspace by the prefixed number first
    /// and if there is none found, it falls back to focusing the workspace by name.
    #[arg(long, group = "focus_by")]
    smart: bool,

    /// Workspace to focus
    workspace: String,
}

#[derive(Debug)]
pub(crate) enum FocusBy {
    Name(String),
    Num(i32),
    Id(i64),
}

impl TryFrom<Focus> for FocusBy {
    type Error = ParseIntError;

    fn try_from(value: Focus) -> Result<Self, Self::Error> {
        if value.name {
            Ok(Self::Name(value.workspace))
        } else if value.num {
            Ok(Self::Num(value.workspace.parse()?))
        } else if value.id {
            Ok(Self::Id(value.workspace.parse()?))
        } else if value.smart {
            if let Ok(num) = value.workspace.parse() {
                return Ok(Self::Num(num));
            }
            Ok(Self::Name(value.workspace))
        } else {
            Ok(Self::Name(value.workspace))
        }
    }
}
