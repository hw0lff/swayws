use colored::*;
use structopt::StructOpt;
use swayipc::Connection;

/// Sway Workspace
/// - operates on sway-workspaces
#[derive(Debug, StructOpt)]
#[structopt(name = "swayws")]
struct SwayWs {
    /// Enable debug mode
    #[structopt(short, long)]
    debug: bool,

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
    List {
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
        /// Focuses specified output
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
    println!("{:?}", opt);

    let mut connection = Connection::new()?;
    let sway_version = connection.get_version().unwrap();
    println!("Sway version: {}", sway_version.human_readable);

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
        Command::Move {
            away,
            focus,
            workspace,
            output,
        } => {
            cmd_move(&mut connection, &output, &workspace, &away);
            dbg!("done", focus);
            if focus == true {
                focus_saved_workspace = false;
            }
        }
        Command::Range {
            away: _,
            start,
            end,
            output,
        } => cmd_range(&mut connection, output, start, end),
        Command::List {
            workspaces,
            outputs,
        } => cmd_list(&mut connection, outputs, workspaces),
        Command::Focus { workspace } => {
            cmd_focus(&mut connection, &workspace);
            focus_saved_workspace = false;
        }
    }

    if let Some(next_workspace) = current_workspace {
        dbg!(focus_saved_workspace);
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
    println!("swayipc-send command: >{}<", &command_text);
    for outcome in connection.run_command(&command_text).unwrap() {
        if outcome.success {
            println!("swayipc-send: success");
        } else {
            println!("swayipc-send: failure '{}'", outcome.error.unwrap());
        }
    }
}

fn cmd_focus(connection: &mut Connection, workspace: &str) {
    focus_workspace(connection, workspace);
}

fn cmd_list(connection: &mut Connection, outputs: bool, workspaces: bool) {
    println!(
        "{}",
        format!(
            "Sway version: {}\noutputs_flag:{}\nworkspaces_flag:{}",
            connection.get_version().unwrap().human_readable,
            outputs,
            workspaces
        )
        .yellow()
    );

    let outputs = connection.get_outputs().unwrap();
    // println!("Outputs:");
    for monitor in outputs.into_iter() {
        println!("{}", monitor.name);
    }
}

fn cmd_move(
    connection: &mut Connection,
    main_output: &str,
    workspace: &str,
    away: &bool,
) {
    if *away {
        let second_output = get_second_output(connection, main_output).unwrap();
        println!("{:?}", second_output);
        move_workspace_to_output(connection, workspace, &second_output.name);
    } else {
        move_workspace_to_output(connection, workspace, main_output);
    }
}

fn cmd_range(connection: &mut Connection, main_output: String, start: String, end: String) {
    println!(
        "{:?}\n{}\n{}\n{}",
        connection.get_version().unwrap().human_readable,
        main_output,
        start,
        end
    );
}

fn get_second_output(
    connection: &mut Connection,
    main_output: &str,
) -> Option<swayipc::reply::Output> {
    let outputs = connection.get_outputs().ok()?;
    if outputs.len() == 1 {
        return None;
    }
    for monitor in outputs.into_iter() {
        println!("{}", monitor.name);
        if monitor.name != main_output {
            return Some(monitor);
        }
    }
    None
}
