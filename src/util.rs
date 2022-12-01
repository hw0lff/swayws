use swayipc::Connection;
use swayipc::Workspace;

use crate::error::SwayWsError;

pub fn focus_workspace(
    connection: &mut Connection,
    workspace_name: &str,
) -> Result<(), SwayWsError> {
    let command_text = format!("workspace {}", workspace_name);
    send_ipc_command(connection, &command_text)
}

pub fn move_workspace_to_output(
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

pub fn send_ipc_command(
    connection: &mut Connection,
    command_text: &str,
) -> Result<(), SwayWsError> {
    let outcomes: Result<(), swayipc::Error> =
        connection.run_command(command_text)?.into_iter().collect();
    Ok(outcomes?)
}

pub fn print_outputs(connection: &mut Connection) -> Result<(), SwayWsError> {
    let outputs = connection.get_outputs()?;
    println!("Outputs (name):");
    for monitor in outputs.into_iter() {
        println!("{}", monitor.name);
    }
    Ok(())
}

pub fn print_workspaces(connection: &mut Connection) -> Result<(), SwayWsError> {
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

pub fn get_second_output(
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

pub fn is_not_in_list<V: Eq>(v: &V, list: &[V]) -> bool {
    for value in list.iter() {
        if *value == *v {
            return false;
        }
    }
    true
}
