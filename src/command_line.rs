//! Command-line processing.

use crate::proptesting::{self, Verbosity};
use crate::simple_reports::{
    run_coarse_subcommand, run_compare_subcommand, run_decl_compare_subcommand,
    run_inspect_subcommand, DetailsMode,
};
use crate::simple_tests::{run_identcheck_subcommand, run_test_subcommand};
use crate::{testcases, CleaningMode, Edition, Lowering, LATEST_EDITION};

const USAGE: &str = "\
Usage: lexeywan [<subcommand>] [...options]

Subcommands:
 *test          [suite-opts]
  compare       [suite-opts] [comparison-opts] [dialect-opts]
  decl-compare  [suite-opts] [comparison-opts] [--edition=2015|2021|*2024]
  inspect       [suite-opts] [dialect-opts]
  coarse        [suite-opts] [dialect-opts]
  identcheck
  proptest      [--count] [--strategy=<name>] [--print-failures|--print-all]
                [dialect-opts]

suite-opts (specify at most one):
  --short: run the SHORTLIST rather than the LONGLIST
  --xfail: run the tests which are expected to fail

dialect-opts:
  --edition=2015|2021|*2024
  --cleaning=none|*shebang|shebang-and-frontmatter
  --lower-doc-comments

comparison-opts:
  --failures-only: don't report cases where the lexers agree
  --details=always|*failures|never

* -- default

";

const DEFAULT_PROPTEST_COUNT: u32 = 5000;

pub fn run_cli() -> impl std::process::Termination {
    match run_cli_impl() {
        Ok(status) => std::process::ExitCode::from(match status {
            SubcommandStatus::Normal => 0,
            SubcommandStatus::ChecksFailed => 3,
        }),
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

pub enum SubcommandStatus {
    Normal,
    ChecksFailed,
}

fn run_cli_impl() -> Result<SubcommandStatus, pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains("--help") {
        print!("{USAGE}");
        return Ok(SubcommandStatus::Normal);
    }

    fn requested_edition(args: &mut pico_args::Arguments) -> Result<Edition, pico_args::Error> {
        Ok(
            match args
                .opt_value_from_str::<_, String>("--edition")?
                .as_deref()
            {
                Some("2015") => Edition::E2015,
                Some("2021") => Edition::E2021,
                Some("2024") => Edition::E2024,
                None => LATEST_EDITION,
                _ => {
                    return Err(pico_args::Error::ArgumentParsingFailed {
                        cause: "unknown edition".into(),
                    })
                }
            },
        )
    }

    fn requested_cleaning_mode(
        args: &mut pico_args::Arguments,
    ) -> Result<CleaningMode, pico_args::Error> {
        Ok(
            match args
                .opt_value_from_str::<_, String>("--cleaning")?
                .as_deref()
            {
                Some("none") => CleaningMode::NoCleaning,
                Some("shebang") => CleaningMode::CleanShebang,
                Some("shebang-and-frontmatter") => CleaningMode::CleanShebangAndFrontmatter,
                None => CleaningMode::CleanShebang,
                _ => {
                    return Err(pico_args::Error::ArgumentParsingFailed {
                        cause: "unknown cleaning mode".into(),
                    })
                }
            },
        )
    }

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
        Test {
            inputs: &'static [&'static str],
        },
        Compare {
            inputs: &'static [&'static str],
            show_failures_only: bool,
            details_mode: DetailsMode,
            edition: Edition,
            cleaning: CleaningMode,
            lowering: Lowering,
        },
        DeclCompare {
            inputs: &'static [&'static str],
            show_failures_only: bool,
            details_mode: DetailsMode,
            edition: Edition,
        },
        Inspect {
            inputs: &'static [&'static str],
            edition: Edition,
            cleaning: CleaningMode,
            lowering: Lowering,
        },
        Coarse {
            inputs: &'static [&'static str],
            edition: Edition,
            cleaning: CleaningMode,
            lowering: Lowering,
        },
        IdentCheck,
        PropTest {
            strategy_name: String,
            count: u32,
            verbosity: Verbosity,
            edition: Edition,
            cleaning: CleaningMode,
            lowering: Lowering,
        },
    }
    fn test_action(args: &mut pico_args::Arguments) -> Result<Action, pico_args::Error> {
        Ok(Action::Test {
            inputs: requested_inputs(args),
        })
    }
    fn compare_action(args: &mut pico_args::Arguments) -> Result<Action, pico_args::Error> {
        let show_failures_only = args.contains("--failures-only");
        Ok(Action::Compare {
            inputs: requested_inputs(args),
            show_failures_only,
            details_mode: requested_details_mode(args)?,
            edition: requested_edition(args)?,
            cleaning: requested_cleaning_mode(args)?,
            lowering: requested_lowering(args),
        })
    }
    fn decl_compare_action(args: &mut pico_args::Arguments) -> Result<Action, pico_args::Error> {
        let show_failures_only = args.contains("--failures-only");
        Ok(Action::DeclCompare {
            inputs: requested_inputs(args),
            show_failures_only,
            details_mode: requested_details_mode(args)?,
            edition: requested_edition(args)?,
        })
    }
    let action = match args.subcommand()?.as_deref() {
        Some("test") => test_action(&mut args)?,
        Some("compare") => compare_action(&mut args)?,
        Some("decl-compare") => decl_compare_action(&mut args)?,
        Some("inspect") => Action::Inspect {
            inputs: requested_inputs(&mut args),
            edition: requested_edition(&mut args)?,
            cleaning: requested_cleaning_mode(&mut args)?,
            lowering: requested_lowering(&mut args),
        },
        Some("coarse") => Action::Coarse {
            inputs: requested_inputs(&mut args),
            edition: requested_edition(&mut args)?,
            cleaning: requested_cleaning_mode(&mut args)?,
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
                edition: requested_edition(&mut args)?,
                cleaning: requested_cleaning_mode(&mut args)?,
                lowering: requested_lowering(&mut args),
            }
        }
        None => test_action(&mut args)?,
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

    Ok(match action {
        Action::Test { inputs } => run_test_subcommand(inputs),
        Action::Compare {
            inputs,
            show_failures_only,
            edition,
            details_mode,
            cleaning,
            lowering,
        } => run_compare_subcommand(
            inputs,
            edition,
            cleaning,
            lowering,
            details_mode,
            show_failures_only,
        ),
        Action::DeclCompare {
            inputs,
            show_failures_only,
            details_mode,
            edition,
        } => run_decl_compare_subcommand(inputs, edition, details_mode, show_failures_only),
        Action::Inspect {
            inputs,
            edition,
            cleaning,
            lowering,
        } => run_inspect_subcommand(inputs, edition, cleaning, lowering),
        Action::Coarse {
            inputs,
            edition,
            cleaning,
            lowering,
        } => run_coarse_subcommand(inputs, edition, cleaning, lowering),
        Action::IdentCheck => run_identcheck_subcommand(),
        Action::PropTest {
            strategy_name,
            count,
            verbosity,
            edition,
            cleaning,
            lowering,
        } => proptesting::run_proptests(
            &strategy_name,
            count,
            verbosity,
            edition,
            cleaning,
            lowering,
        ),
    })
}
