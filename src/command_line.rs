//! Command-line processing.

use crate::proptesting::{self, Verbosity};
use crate::simple_reports::{
    run_coarse_subcommand, run_compare_subcommand, run_decl_compare_subcommand,
    run_identcheck_subcommand, run_inspect_subcommand, DetailsMode,
};
use crate::{testcases, Edition, Lowering};

const USAGE: &str = "\
Usage: lexeywan [global-opts] [<subcommand>] [...options]

global-opts:
  --edition=2015|2021|*2024

Subcommands:
 *compare       [suite-opts] [comparison-opts] [--lower-doc-comments]
  decl-compare  [suite-opts] [comparison-opts]
  inspect       [suite-opts] [--lower-doc-comments]
  coarse        [suite-opts] [--lower-doc-comments]
  identcheck
  proptest      [--count] [--strategy=<name>] [--print-failures|--print-all]
                [--lower-doc-comments]

suite-opts (specify at most one):
  --short: run the SHORTLIST rather than the LONGLIST
  --xfail: run the tests which are expected to to fail

comparison-opts:
  --failures-only: don't report cases where the lexers agree
  --details=always|*failures|never

* -- default

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
        print!("{USAGE}");
        return Ok(());
    }

    let edition = match args
        .opt_value_from_str::<_, String>("--edition")?
        .as_deref()
    {
        Some("2015") => Edition::E2015,
        Some("2021") => Edition::E2021,
        Some("2024") => Edition::E2024,
        None => Edition::E2024,
        _ => {
            return Err(pico_args::Error::ArgumentParsingFailed {
                cause: "unknown edition".into(),
            })
        }
    };

    fn requested_lowering(args: &mut pico_args::Arguments) -> Lowering {
        if args.contains("--lower-doc-comments") {
            Lowering::LowerDocComments
        } else {
            Lowering::NoLowering
        }
    }

    fn requested_inputs(args: &mut pico_args::Arguments) -> &'static [&'static str] {
        if args.contains("--short") {
            testcases::SHORTLIST
        } else if args.contains("--xfail") {
            testcases::XFAIL
        } else {
            testcases::LONGLIST
        }
    }

    fn requested_details_mode(
        args: &mut pico_args::Arguments,
    ) -> Result<DetailsMode, pico_args::Error> {
        Ok(
            match args
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
            },
        )
    }

    enum Action {
        Compare {
            inputs: &'static [&'static str],
            show_failures_only: bool,
            details_mode: DetailsMode,
            lowering: Lowering,
        },
        DeclCompare {
            inputs: &'static [&'static str],
            show_failures_only: bool,
            details_mode: DetailsMode,
        },
        Inspect {
            inputs: &'static [&'static str],
            lowering: Lowering,
        },
        Coarse {
            inputs: &'static [&'static str],
            lowering: Lowering,
        },
        IdentCheck,
        PropTest {
            strategy_name: String,
            count: u32,
            verbosity: Verbosity,
            lowering: Lowering,
        },
    }
    fn compare_action(args: &mut pico_args::Arguments) -> Result<Action, pico_args::Error> {
        let show_failures_only = args.contains("--failures-only");
        Ok(Action::Compare {
            inputs: requested_inputs(args),
            show_failures_only,
            details_mode: requested_details_mode(args)?,
            lowering: requested_lowering(args),
        })
    }
    fn decl_compare_action(args: &mut pico_args::Arguments) -> Result<Action, pico_args::Error> {
        let show_failures_only = args.contains("--failures-only");
        Ok(Action::DeclCompare {
            inputs: requested_inputs(args),
            show_failures_only,
            details_mode: requested_details_mode(args)?,
        })
    }
    let action = match args.subcommand()?.as_deref() {
        Some("compare") => compare_action(&mut args)?,
        Some("decl-compare") => decl_compare_action(&mut args)?,
        Some("inspect") => Action::Inspect {
            inputs: requested_inputs(&mut args),
            lowering: requested_lowering(&mut args),
        },
        Some("coarse") => Action::Coarse {
            inputs: requested_inputs(&mut args),
            lowering: requested_lowering(&mut args),
        },
        Some("identcheck") => Action::IdentCheck,
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
                lowering: requested_lowering(&mut args),
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
            lowering,
        } => run_compare_subcommand(inputs, edition, lowering, details_mode, show_failures_only),
        Action::DeclCompare {
            inputs,
            show_failures_only,
            details_mode,
        } => run_decl_compare_subcommand(inputs, edition, details_mode, show_failures_only),
        Action::Inspect { inputs, lowering } => run_inspect_subcommand(inputs, edition, lowering),
        Action::Coarse { inputs, lowering } => run_coarse_subcommand(inputs, edition, lowering),
        Action::IdentCheck => run_identcheck_subcommand(edition),
        Action::PropTest {
            strategy_name,
            count,
            verbosity,
            lowering,
        } => proptesting::run_proptests(&strategy_name, count, verbosity, edition, lowering),
    }

    Ok(())
}
