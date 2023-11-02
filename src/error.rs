use snafu::prelude::*;
use snafu::Location;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Ctx)))]
#[snafu(visibility(pub))]
pub enum SwayWsError {
    #[snafu(display("[{location}] Error while communicating with sway"))]
    SwayIpc {
        source: swayipc::Error,
        location: Location,
    },

    #[snafu(display("[{location}] Cannot parse integer"))]
    Parse {
        source: std::num::ParseIntError,
        location: Location,
    },

    #[snafu(display("[{location}] No output can be matched against the specified parameters"))]
    NoOutputMatched { location: Location },
}

pub(crate) fn report(error: &dyn snafu::Error) -> String {
    let sources = snafu::ChainCompat::new(error);
    let sources: Vec<&dyn snafu::Error> = sources.collect();
    let sources = sources.iter().rev();
    let mut s = String::new();
    for (i, source) in sources.enumerate() {
        s = match i {
            0 => format!("{source}"),
            _ => format!("{source} ({s})"),
        }
    }
    s
}
