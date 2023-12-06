use std::convert::TryInto;
use std::str::FromStr;

use clap::Parser;
use snafu::prelude::*;
use swayipc::Connection;
use swayipc::Workspace;

mod args;
mod cmd_virtual;
mod error;
mod util;

use args::{Command, FocusBy, SwayWs};
use error::*;
use util::*;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    if let Err(err) = run() {
        log::error!("{}", report(&err));
    }
}

fn run() -> Result<(), SwayWsError> {
    let opt: SwayWs = SwayWs::parse();

    let mut connection = Connection::new().context(SwayIpcCtx)?;

    let workspaces = connection.get_workspaces().context(SwayIpcCtx)?;

    let mut previously_visible_workspaces: Vec<String> = vec![];
    let mut previously_focused_workspace: Option<String> = None;
    let mut restore_visible_workspaces: bool = true;

    // Store the currently visible and focused workspaces
    // Note: n > 0 workspaces can be visible but only 1 workspace can be focused
    for ws in workspaces.into_iter() {
        if ws.visible {
            previously_visible_workspaces.push(ws.name.clone());
        }
        if ws.focused {
            previously_focused_workspace = Some(ws.name);
        }
    }

    match opt.cmd {
        Command::Focus(focus) => {
            let focus_by = focus.try_into().context(ParseCtx)?;
            cmd_focus(&mut connection, focus_by)?;
            restore_visible_workspaces = false;
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
                restore_visible_workspaces = false;
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
            restore_visible_workspaces = false;
        }
        Command::Swap { ws_l, ws_r } => {
            cmd_swap(&mut connection, ws_l.clone(), ws_r.clone())?;
            if previously_focused_workspace == Some(ws_l.clone()) {
                previously_focused_workspace = Some(ws_r)
            } else if previously_focused_workspace == Some(ws_r) {
                previously_focused_workspace = Some(ws_l)
            }
        }
        Command::Virtual(v) => cmd_virtual::cmd_virtual(v, &mut connection)?,
    }

    // Make the same workspaces visible again that were visible before rearranging the
    // workspace-to-output mapping and focus the previously focused workspace
    if restore_visible_workspaces {
        // First, visit all previously visible workspaces by focusing them
        for ws_name in previously_visible_workspaces {
            focus_workspace(&mut connection, &ws_name)?;
        }
        // At last, focus the saved workspace
        if let Some(ws_name) = previously_focused_workspace {
            focus_workspace(&mut connection, &ws_name)?;
        }
    }

    Ok(())
}

fn cmd_focus(connection: &mut Connection, focus_by: FocusBy) -> Result<(), SwayWsError> {
    match focus_by {
        FocusBy::Name(name) => focus_workspace_by_name(connection, &name),
        FocusBy::Num(num) => focus_workspace_by_num(connection, num),
        FocusBy::Id(id) => focus_workspace_by_id(connection, id),
    }
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
        let start_i: i32 = i32::from_str(start).context(ParseCtx)?;
        let end_i: i32 = i32::from_str(end).context(ParseCtx)?;

        for i in start_i..=end_i {
            cmd_move(connection, output_name, &i.to_string(), away, &not)?;
        }

        return Ok(());
    }

    let mut ws_list: Vec<String> = vec![];
    let mut fill_ws_list: bool = false;

    // collect workspaces between start and end in a vector
    let workspaces: Vec<Workspace> = connection.get_workspaces().context(SwayIpcCtx)?;
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

fn cmd_swap(connection: &mut Connection, ws_l: String, ws_r: String) -> Result<(), SwayWsError> {
    let tmp = "swayws-swap";
    let o_l = connection
        .get_workspaces()
        .context(SwayIpcCtx)?
        .iter()
        .find(|ws| ws.name == ws_l)
        .cloned();
    let o_r = connection
        .get_workspaces()
        .context(SwayIpcCtx)?
        .iter()
        .find(|ws| ws.name == ws_r)
        .cloned();

    rename_workspace(connection, &ws_l, tmp)?;
    rename_workspace(connection, &ws_r, &ws_l)?;
    rename_workspace(connection, tmp, &ws_r)?;

    if let Some(o_l) = o_l {
        move_workspace_to_output(connection, &ws_l, &o_l.output)?;
    }
    if let Some(o_r) = o_r {
        move_workspace_to_output(connection, &ws_r, &o_r.output)?;
    }

    Ok(())
}
