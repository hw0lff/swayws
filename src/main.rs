use std::str::FromStr;

use clap::{Parser, Subcommand};
use swayipc::Connection;
use swayipc::Workspace;

mod error;
mod util;
use error::SwayWsError;
use util::*;

/// Sway Workspace
/// - operates on sway workspaces
#[derive(Debug, Parser)]
#[clap(name = "swayws", version)]
struct SwayWs {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
}

fn main() {
    env_logger::builder().format_timestamp(None).init();

    if let Err(err) = run() {
        log::error!("{}", err);
    }
}

fn run() -> Result<(), SwayWsError> {
    let opt: SwayWs = SwayWs::parse();

    let mut connection = Connection::new()?;

    let workspaces = connection.get_workspaces()?;

    let mut current_workspace: Option<String> = None;
    let mut focus_saved_workspace: bool = true;

    for ws in workspaces.into_iter() {
        if ws.focused {
            current_workspace = Some(ws.name);
        }
    }

    match opt.cmd {
        Command::Focus { workspace } => {
            cmd_focus(&mut connection, &workspace)?;
            focus_saved_workspace = false;
        }
        Command::Move {
            away,
            not,
            focus,
            workspace,
            output,
        } => {
            cmd_move(&mut connection, &output, &workspace, &away, &not)?;
            if focus {
                focus_saved_workspace = false;
            }
        }
        Command::Range {
            away,
            not,
            numeric,
            start,
            end,
            output,
        } => cmd_range(&mut connection, &output, &start, &end, &away, &numeric, not)?,
        Command::List {
            workspaces,
            outputs,
        } => {
            cmd_list(&mut connection, outputs, workspaces)?;
        }
    }

    if let Some(next_workspace) = current_workspace {
        if focus_saved_workspace {
            focus_workspace(&mut connection, &next_workspace)?;
        }
    }

    Ok(())
}

fn cmd_focus(connection: &mut Connection, workspace: &str) -> Result<(), SwayWsError> {
    focus_workspace(connection, workspace)
}

fn cmd_list(
    connection: &mut Connection,
    outputs: bool,
    workspaces: bool,
) -> Result<(), SwayWsError> {
    if outputs {
        print_outputs(connection)?;
    }
    if workspaces {
        print_workspaces(connection)?;
    }
    if !outputs && !workspaces {
        print_outputs(connection)?;
        print_workspaces(connection)?;
    }
    Ok(())
}

fn cmd_move(
    connection: &mut Connection,
    output_name: &str,
    workspace: &str,
    away: &bool,
    not: &Option<Vec<String>>,
) -> Result<(), SwayWsError> {
    if *away {
        let second_output = match not {
            None => get_second_output(connection, &[output_name.into()])?,
            Some(not_list) => {
                let mut list = vec![output_name.into()];
                list.append(&mut not_list.clone());

                get_second_output(connection, &list)?
            }
        };
        move_workspace_to_output(connection, workspace, &second_output.name)?;
    } else {
        move_workspace_to_output(connection, workspace, output_name)?;
    }
    Ok(())
}

fn cmd_range(
    connection: &mut Connection,
    output_name: &str,
    start: &str,
    end: &str,
    away: &bool,
    numeric: &bool,
    not: Option<Vec<String>>,
) -> Result<(), SwayWsError> {
    if *numeric {
        let start_i: i32 = i32::from_str(start)?;
        let end_i: i32 = i32::from_str(end)?;

        for i in start_i..=end_i {
            cmd_move(connection, output_name, &i.to_string(), away, &not)?;
        }

        return Ok(());
    }

    let mut ws_list: Vec<String> = vec![];
    let mut fill_ws_list: bool = false;

    // collect workspaces between start and end in a vector
    let workspaces: Vec<Workspace> = connection.get_workspaces()?;
    for ws in workspaces.into_iter() {
        if start.cmp(&ws.name).is_eq() {
            fill_ws_list = true;
        }
        if fill_ws_list {
            ws_list.push(ws.name.clone());
        }
        if end.cmp(&ws.name).is_eq() {
            fill_ws_list = false;
        }
    }

    for ws in ws_list.into_iter() {
        cmd_move(connection, output_name, &ws, away, &not)?;
    }
    Ok(())
}
