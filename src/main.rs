use structopt::StructOpt;
use swayipc::reply::Workspace;
use swayipc::Connection;

/// Sway Workspace
/// - operates on sway workspaces
#[derive(Debug, StructOpt)]
#[structopt(name = "swayws")]
struct SwayWs {
    /// Use verbose output
    #[structopt(short, parse(from_occurrences))]
    verbose: u8,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
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
        #[structopt(short = "o", long)]
        outputs: bool,

        /// List workspaces
        #[structopt(short = "ws", long)]
        workspaces: bool,
    },
    /// Moves a workspace to a specified output
    Move {
        /// Moves workspace to output that does not match the specified output name
        #[structopt(short, long)]
        away: bool,
        /// Focuses specified workspace
        #[structopt(short, long)]
        focus: bool,

        /// Workspace to move to the other monitor
        #[structopt()]
        workspace: String,

        /// Name of the output
        #[structopt()]
        output: String,
    },
    /// Moves a range of workspaces to a specified output
    Range {
        /// Moves workspace to output that does not match the specified output name
        #[structopt(short, long)]
        away: bool,

        /// First workspace in range
        #[structopt()]
        start: String,
        /// Last workspace in range
        #[structopt()]
        end: String,

        /// Name of the output
        #[structopt()]
        output: String,
    },
}

fn main() -> Result<(), swayipc::Error> {
    // let mut connection = match Connection::new() {
    //     Ok(connection) => connection,
    //     Err(sway_ipc_error) => panic!("{:?}", sway_ipc_error),
    // };

    let opt: SwayWs = SwayWs::from_args();
    // println!("{:?}", opt);

    let mut connection = Connection::new()?;
    // let sway_version = connection.get_version().unwrap();
    // println!("Sway version: {}", sway_version.human_readable);

    let workspaces = connection.get_workspaces().unwrap();

    let mut current_workspace: Option<String> = None;
    let mut focus_saved_workspace: bool = true;

    for ws in workspaces.into_iter() {
        // println!("{}: {}", ws.name, ws.focused);
        if ws.focused {
            current_workspace = Some(ws.name);
        }
    }

    match opt.cmd {
        Command::Focus { workspace } => {
            cmd_focus(&mut connection, &workspace);
            focus_saved_workspace = false;
        }
        Command::Move {
            away,
            focus,
            workspace,
            output,
        } => {
            cmd_move(&mut connection, &output, &workspace, &away);
            if focus {
                focus_saved_workspace = false;
            }
        }
        Command::Range {
            away,
            start,
            end,
            output,
        } => {
            cmd_range(&mut connection, &output, &start, &end, &away);
        }
        Command::List {
            workspaces,
            outputs,
        } => {
            cmd_list(&mut connection, outputs, workspaces);
        }
    }

    if let Some(next_workspace) = current_workspace {
        // dbg!(focus_saved_workspace);
        if focus_saved_workspace {
            focus_workspace(&mut connection, &next_workspace);
        }
    }

    Ok(())
}

fn focus_workspace(connection: &mut Connection, workspace_name: &str) {
    let command_text = format!("workspace {}", workspace_name);
    send_ipc_command(connection, &command_text);
}

fn move_workspace_to_output(connection: &mut Connection, workspace_name: &str, output_name: &str) {
    let command_text = format!(
        "workspace {0} output {1},\
        workspace {0},\
        move workspace to {1}",
        workspace_name, output_name
    );
    send_ipc_command(connection, &command_text);
}

fn send_ipc_command(connection: &mut Connection, command_text: &str) {
    // println!("swayipc-send command: >{}<", &command_text);
    for outcome in connection.run_command(&command_text).unwrap() {
        if outcome.success {
            // println!("swayipc-send: success");
        } else {
            // println!("swayipc-send: failure '{}'", outcome.error.unwrap());
        }
    }
}

fn cmd_focus(connection: &mut Connection, workspace: &str) {
    focus_workspace(connection, workspace);
}

fn cmd_list(connection: &mut Connection, outputs: bool, workspaces: bool) {
    // println!(
    //     "{}",
    //     format!(
    //         "Sway version: {}\noutputs_flag:{}\nworkspaces_flag:{}",
    //         connection.get_version().unwrap().human_readable,
    //         outputs,
    //         workspaces
    //     )
    //     .yellow()
    // );

    fn print_outputs(connection: &mut Connection) {
        let outputs = connection.get_outputs().unwrap();
        println!("Outputs (name):");
        for monitor in outputs.into_iter() {
            println!("{}", monitor.name);
        }
    }

    fn print_workspaces(connection: &mut Connection) {
        let workspaces: Vec<Workspace> = connection.get_workspaces().unwrap();
        println!("Workspaces (id, name):");
        let fill = workspaces.last().unwrap().num.to_string().len();
        for ws in workspaces.into_iter() {
            println!("{0:>width$} {1:>width$}", ws.num, ws.name, width = fill);
        }
    }

    match (outputs, workspaces) {
        (false, false) => {
            print_outputs(connection);
            print_workspaces(connection);
        }
        (false, true) => {
            print_workspaces(connection);
        }
        (true, false) => {
            print_outputs(connection);
        }
        (true, true) => {
            print_outputs(connection);
            print_workspaces(connection);
        }
    }
}

fn cmd_move(connection: &mut Connection, output_name: &str, workspace: &str, away: &bool) {
    if *away {
        let second_output = get_second_output(connection, output_name).unwrap();
        // println!("{:?}", second_output);
        move_workspace_to_output(connection, workspace, &second_output.name);
    } else {
        move_workspace_to_output(connection, workspace, output_name);
    }
}

fn cmd_range(connection: &mut Connection, output_name: &str, start: &str, end: &str, away: &bool) {
    let mut ws_list: Vec<String> = vec![];
    let mut fill_ws_list: bool = false;

    // collect workspaces between start and end in a vector
    let workspaces: Vec<Workspace> = connection.get_workspaces().unwrap();
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
        cmd_move(connection, &output_name, &ws, &away);
    }
}

fn get_second_output(
    connection: &mut Connection,
    output_name: &str,
) -> Option<swayipc::reply::Output> {
    let outputs = connection.get_outputs().ok()?;
    if outputs.len() == 1 {
        return None;
    }
    for monitor in outputs.into_iter() {
        // println!("{}", monitor.name);
        if monitor.name != output_name {
            return Some(monitor);
        }
    }
    None
}
