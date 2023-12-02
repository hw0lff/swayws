use clap::{Parser, Subcommand};

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
    Focus {
        /// Workspace to focus
        workspace: String,
    },
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
