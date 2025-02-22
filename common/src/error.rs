use std::backtrace::BacktraceStatus;

use anyhow::Error;
use simplelog::error;

pub struct FancyError();

impl FancyError {
    pub fn print_fancy(error: &Error, critical: bool) {
        let exit_message = if critical {
            "An error occurred causing the application to exit. The application cannot continue after this error."
        } else {
            "An error occurred, but the application can continue. The application may not function as expected."
        };

        error!("{}", exit_message);
        error!("If you believe this error was not caused by the runtime, for example: a missing network connection, please report this error to the developers.");
        error!("Create a new issue on the GitHub repository at the following link: https://github.com/HttpRafa/atomic-cloud with the information below:");

        error!("Error: {}", error);
        error
            .chain()
            .skip(1)
            .for_each(|error| error!("    Caused by: {}", error));

        match error.backtrace().status() {
            BacktraceStatus::Captured => {
                error!("Backtrace:");
                format!("{}", error.backtrace())
                    .lines()
                    .for_each(|line| error!("{}", line));
            }
            _ => {
                error!("Backtrace is not available. Ensure you run the program with `RUST_BACKTRACE=1` to enable backtraces.");
            }
        }
    }
}
