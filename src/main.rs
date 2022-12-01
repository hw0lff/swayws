use std::str::FromStr;

use clap::{Parser, Subcommand};
use swayipc::Connection;
use swayipc::Workspace;

mod error;
use error::SwayWsError;

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
    Focus {
        /// Workspace to focus
        workspace: String,
    },
    /// Lists infos about sway
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
    Move {
        /// Moves workspace to output that does not match the specified output name
        #[clap(short, long)]
        away: bool,
        /// Excludes outputs to move workspace to,
        /// has to be used with --away
        #[clap(long, requires("away"))]
        not: Option<Vec<String>>,
        /// Focuses specified workspace
        #[clap(short, long)]
        focus: bool,

        /// Workspace to move
        workspace: String,

        /// Name of the output
        output: String,
    },
    /// Moves a range of workspaces to a specified output
    Range {
        /// Moves workspace to output that does not match the specified output name
        #[clap(short, long)]
        away: bool,
        /// Excludes outputs to move workspace to,
        /// has to be used with --away
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

fn focus_workspace(connection: &mut Connection, workspace_name: &str) -> Result<(), SwayWsError> {
    let command_text = format!("workspace {}", workspace_name);
    send_ipc_command(connection, &command_text)
}

fn move_workspace_to_output(
    connection: &mut Connection,
    workspace_name: &str,
    output_name: &str,
) -> Result<(), SwayWsError> {
    let command_text = format!(
        "workspace {0} output {1},\
        workspace {0},\
        move workspace to {1}",
        workspace_name, output_name
    );
    send_ipc_command(connection, &command_text)
}

fn send_ipc_command(connection: &mut Connection, command_text: &str) -> Result<(), SwayWsError> {
    let outcomes: Result<(), swayipc::Error> =
        connection.run_command(command_text)?.into_iter().collect();
    Ok(outcomes?)
}

fn cmd_focus(connection: &mut Connection, workspace: &str) -> Result<(), SwayWsError> {
    focus_workspace(connection, workspace)
}

fn cmd_list(
    connection: &mut Connection,
    outputs: bool,
    workspaces: bool,
) -> Result<(), SwayWsError> {
    fn print_outputs(connection: &mut Connection) -> Result<(), SwayWsError> {
        let outputs = connection.get_outputs()?;
        println!("Outputs (name):");
        for monitor in outputs.into_iter() {
            println!("{}", monitor.name);
        }
        Ok(())
    }

    fn print_workspaces(connection: &mut Connection) -> Result<(), SwayWsError> {
        let workspaces: Vec<Workspace> = connection.get_workspaces()?;
        println!("Workspaces (id, name):");
        let fill = match workspaces.last() {
            Some(ws) => ws.num.to_string().len(),
            None => 1,
        };
        for ws in workspaces.into_iter() {
            println!("{0:>width$} {1:>width$}", ws.num, ws.name, width = fill);
        }
        Ok(())
    }

    match (outputs, workspaces) {
        (false, false) => {
            print_outputs(connection)?;
            print_workspaces(connection)?;
        }
        (false, true) => {
            print_workspaces(connection)?;
        }
        (true, false) => {
            print_outputs(connection)?;
        }
        (true, true) => {
            print_outputs(connection)?;
            print_workspaces(connection)?;
        }
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

fn get_second_output(
    connection: &mut Connection,
    output_names: &[String],
) -> Result<swayipc::Output, SwayWsError> {
    let outputs = connection.get_outputs()?;
    if outputs.len() == 1 {
        return Err(SwayWsError::NoOutputMatched);
    }
    outputs
        .into_iter()
        .find(|monitor| is_not_in_list(&monitor.name, output_names))
        .ok_or(SwayWsError::NoOutputMatched)
}

fn is_not_in_list<V: Eq>(v: &V, list: &[V]) -> bool {
    for value in list.iter() {
        if *value == *v {
            return false;
        }
    }
    true
}
