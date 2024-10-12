//! Command-line processing.

use crate::proptesting::{self, Verbosity};
use crate::simple_reports::{
    run_coarse_subcommand, run_compare_subcommand, run_inspect_subcommand, DetailsMode,
};
use crate::testcases;
use crate::Edition;

const USAGE: &str = "\
Usage: lexeywan [--edition=2015|2021|2024] [<subcommand>] [...options]

Subcommands:
 *compare  [--short] [--failures-only] [--details=always|*failures|never]
  inspect  [--short]
  coarse   [--short]
  proptest [--count] [--strategy=<name>] [--print-failures|--print-all]

* -- default

--short: run the SHORTLIST rather than the LONGLIST

";

const DEFAULT_PROPTEST_COUNT: u32 = 5000;

pub fn run_cli() -> impl std::process::Termination {
    match run_cli_impl() {
        Ok(_) => std::process::ExitCode::from(0),
        Err(pico_args::Error::ArgumentParsingFailed { cause }) => {
            eprint!("{USAGE}{cause}\n");
            std::process::ExitCode::from(2)
        }
        Err(e) => {
            eprint!("{USAGE}{e}\n");
            std::process::ExitCode::from(2)
        }
    }
}
fn run_cli_impl() -> Result<(), pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains("--help") {
        print!("{}", USAGE);
        return Ok(());
    }

    let edition = match args
        .opt_value_from_str::<_, String>("--edition")?
        .as_deref()
    {
        Some("2015") => Edition::E2015,
        Some("2021") => Edition::E2021,
        Some("2024") => Edition::E2024,
        None => Edition::E2021,
        _ => {
            return Err(pico_args::Error::ArgumentParsingFailed {
                cause: "unknown edition".into(),
            })
        }
    };

    fn requested_inputs(args: &mut pico_args::Arguments) -> &'static [&'static str] {
        if args.contains("--short") {
            testcases::SHORTLIST
        } else {
            testcases::LONGLIST
        }
    }

    enum Action {
        Compare {
            inputs: &'static [&'static str],
            show_failures_only: bool,
            details_mode: DetailsMode,
        },
        Inspect {
            inputs: &'static [&'static str],
        },
        Coarse {
            inputs: &'static [&'static str],
        },
        PropTest {
            strategy_name: String,
            count: u32,
            verbosity: Verbosity,
        },
    }
    fn compare_action(args: &mut pico_args::Arguments) -> Result<Action, pico_args::Error> {
        let show_failures_only = args.contains("--failures-only");
        let details_mode = match args
            .opt_value_from_str::<_, String>("--details")?
            .as_deref()
        {
            Some("always") => DetailsMode::Always,
            Some("failures-only") => DetailsMode::Failures,
            Some("never") => DetailsMode::Never,
            None => DetailsMode::Failures,
            _ => {
                return Err(pico_args::Error::ArgumentParsingFailed {
                    cause: "unknown details mode".into(),
                })
            }
        };
        Ok(Action::Compare {
            inputs: requested_inputs(args),
            show_failures_only,
            details_mode,
        })
    }
    let action = match args.subcommand()?.as_deref() {
        Some("compare") => compare_action(&mut args)?,
        Some("inspect") => Action::Inspect {
            inputs: requested_inputs(&mut args),
        },
        Some("coarse") => Action::Coarse {
            inputs: requested_inputs(&mut args),
        },
        Some("proptest") => {
            let strategy_name = args
                .opt_value_from_str::<_, String>("--strategy")?
                .unwrap_or(proptesting::DEFAULT_STRATEGY.into());
            if !proptesting::strategy_names()
                .iter()
                .any(|s| *s == strategy_name)
            {
                return Err(pico_args::Error::ArgumentParsingFailed {
                    cause: format!(
                        "unknown strategy; choose from {}",
                        proptesting::strategy_names().join(",")
                    ),
                });
            }
            let count = args
                .opt_value_from_str::<_, u32>("--count")?
                .unwrap_or(DEFAULT_PROPTEST_COUNT);
            let verbosity = if args.contains("--print-all") {
                Verbosity::PrintAll
            } else if args.contains("--print-failures") {
                Verbosity::PrintFailures
            } else {
                Verbosity::Quiet
            };
            Action::PropTest {
                strategy_name,
                count,
                verbosity,
            }
        }
        None => compare_action(&mut args)?,
        _ => {
            return Err(pico_args::Error::ArgumentParsingFailed {
                cause: "unknown subcommand".into(),
            })
        }
    };

    if !args.finish().is_empty() {
        return Err(pico_args::Error::ArgumentParsingFailed {
            cause: "unknown option".into(),
        });
    }

    match action {
        Action::Compare {
            inputs,
            show_failures_only,
            details_mode,
        } => run_compare_subcommand(inputs, edition, details_mode, show_failures_only),
        Action::Inspect { inputs } => run_inspect_subcommand(inputs, edition),
        Action::Coarse { inputs } => run_coarse_subcommand(inputs, edition),
        Action::PropTest {
            strategy_name,
            count,
            verbosity,
        } => proptesting::run_proptests(&strategy_name, count, verbosity, edition),
    }

    Ok(())
}
