use snafu::prelude::*;
use swayipc::Connection;

use crate::args::{Virtual, VirtualCreateArgs, VirtualUnplugArgs};
use crate::error::{SwayIpcCtx, SwayWsError};
use crate::util::send_ipc_command;

pub(crate) fn cmd_virtual(v: Virtual, connection: &mut Connection) -> Result<(), SwayWsError> {
    match v {
        Virtual::Create(vca) => virtual_create(vca, connection),
        Virtual::Unplug(vua) => virtual_unplug(vua, connection),
    }
}

fn virtual_unplug(vua: VirtualUnplugArgs, connection: &mut Connection) -> Result<(), SwayWsError> {
    if vua.all {
        return unplug_all(connection);
    }
    Ok(())
}

fn unplug_all(connection: &mut Connection) -> Result<(), SwayWsError> {
    connection
        .get_outputs()
        .context(SwayIpcCtx)?
        .into_iter()
        .filter_map(|o| o.name.starts_with("HEADLESS-").then_some(o.name))
        .try_for_each(|name| send_ipc_command(connection, &format!("output {name} unplug")))
}

fn virtual_create(vca: VirtualCreateArgs, connection: &mut Connection) -> Result<(), SwayWsError> {
    let mut suffixes = vca.with_suffixes;
    suffixes.sort();
    let expected_suffixes = suffixes.clone();

    let existing_outputs = connection.get_outputs().context(SwayIpcCtx)?;
    existing_outputs
        .iter()
        .filter_map(|o| o.name.starts_with("HEADLESS-").then_some(o.name.clone()))
        .for_each(|name| suffixes.retain(|suffix| !name.ends_with(&suffix.to_string())));

    let expected_suffixes = expected_suffixes.iter().map(|s| s.to_string());
    for expected_suffix in expected_suffixes {
        let mut first_round = true;
        while !have_output_with_suffix(&expected_suffix, connection)? {
            if !first_round {
                if let Some(latest_output) = connection.get_outputs().context(SwayIpcCtx)?.last() {
                    // unplug latest
                    send_ipc_command(connection, &format!("output {} unplug", latest_output.name))?
                }
            }
            send_ipc_command(connection, "create_output")?;
            first_round = false;
        }
    }
    Ok(())
}

fn have_output_with_suffix(suffix: &str, connection: &mut Connection) -> Result<bool, SwayWsError> {
    Ok(connection
        .get_outputs()
        .context(SwayIpcCtx)?
        .into_iter()
        .filter_map(|o| o.name.starts_with("HEADLESS-").then_some(o.name))
        .any(|name| name.ends_with(suffix)))
}
