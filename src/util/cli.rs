use crate::runners::{ExperimentRunnerArgs, RunMode, SimulationRunnerArgs, SimulationUiRunnerArgs};
use clap::{Arg, ArgAction, Command};

pub fn parse_cli_args() -> RunMode {
    let iterations_arg = Arg::new("iterations")
        .short('i')
        .long("--iterations")
        .help("Specifies how many ticks to execute")
        .action(ArgAction::Set)
        .number_of_values(1);

    let chemistry_key_arg = Arg::new("chemistry_key")
        .short('c')
        .long("--chemistry")
        .help("A key that selects a chemistry")
        .action(ArgAction::Set)
        .number_of_values(1);

    let sim_scenario_key_arg = Arg::new("sim_scenario_key")
        .short('s')
        .long("scenario")
        .help("A key that selects a predefined simulation configuration (required)")
        .action(ArgAction::Set)
        .number_of_values(1);

    let exp_scenario_key_arg = Arg::new("exp_scenario_key")
        .short('s')
        .long("scenario")
        .help("A key that selects a predefined experiment configuration (required)")
        .action(ArgAction::Set)
        .number_of_values(1);

    let exp_name_key_arg = Arg::new("exp_scenario_key")
        .short('n')
        .long("name")
        .help("Specifies a unique key for this specific run of the experiment")
        .action(ArgAction::Set)
        .number_of_values(1);

    let matches = Command::new("proto-molecule")
        .about("A framework for evolving 2d agents")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("exp")
                .about("Run an experiment")
                .arg(exp_scenario_key_arg.clone())
                .arg(exp_name_key_arg.clone()),
        )
        .subcommand(
            Command::new("sim")
                .about("Run a single simulation")
                .arg(chemistry_key_arg.clone())
                .arg(sim_scenario_key_arg.clone())
                .arg(iterations_arg.clone()),
        )
        .subcommand(
            Command::new("sim_ui")
                .about("Run a single simulation")
                .arg(chemistry_key_arg.clone())
                .arg(sim_scenario_key_arg.clone())
                .arg(
                    Arg::new("sim_ticks_per_second")
                        .short('t')
                        .long("tps")
                        .help("A key that selects a predefined simulation configuration (required)")
                        .action(ArgAction::Set)
                        .number_of_values(1),
                )
                .arg(
                    Arg::new("ui_frame_rate")
                        .short('F')
                        .long("frame_rate")
                        .help("A key that selects a predefined simulation configuration (required)")
                        .action(ArgAction::Set)
                        .number_of_values(1),
                )
                .arg(iterations_arg.clone()),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("exp", matches)) => {
            let scenario_key = matches
                .get_one::<String>("scenario_key")
                .expect("Experiment scenario key required");
            let iterations = matches.get_one::<u64>("iterations");

            let default_name_key = "default".to_string();
            let name_key = matches
                .get_one::<String>("name_key")
                .unwrap_or(&default_name_key);

            return RunMode::HeadlessExperiment(ExperimentRunnerArgs {
                experiment_scenario_key: scenario_key.clone(),
                experiment_name_key: name_key.clone(),
            });
        }
        Some(("sim", sim_matches)) => {
            let chemistry_key = sim_matches
                .get_one::<String>("chemistry_key")
                .expect("chemistry key required");

            let sim_scenario_key = sim_matches
                .get_one::<String>("sim_scenario_key")
                .expect("Scenario key required");
            let iterations = sim_matches.get_one::<u64>("iterations");

            let args = SimulationRunnerArgs {
                chemistry_key: chemistry_key.clone(),
                simulation_scenario_key: sim_scenario_key.clone(),
                unit_entry_scenario_key: None,
                iterations: iterations.map(|i| *i),
            };

            return RunMode::HeadlessSimulation(args);
        }
        Some(("sim_ui", sim_matches)) => {
            let sim_scenario_key = sim_matches
                .get_one::<String>("sim_scenario_key")
                .expect("Scenario key required");

            let mut sim_ticks_per_second = sim_matches
                .get_one::<String>("sim_ticks_per_second")
                .map(|x| x.parse::<u32>().unwrap());
            let mut ui_frame_rate = sim_matches
                .get_one::<String>("ui_frame_rate")
                .map(|x| x.parse::<u32>().unwrap());
            let iterations = sim_matches.get_one::<u64>("iterations");

            // the tps should be at least the frame rate
            sim_ticks_per_second = sim_ticks_per_second.or(ui_frame_rate);

            let chemistry_key = sim_matches
                .get_one::<String>("chemistry_key")
                .expect("chemistry key required");

            let args = SimulationRunnerArgs {
                chemistry_key: chemistry_key.clone(),
                simulation_scenario_key: sim_scenario_key.clone(),
                unit_entry_scenario_key: None,
                iterations: iterations.map(|i| *i),
            };

            return RunMode::GuiSimulation(
                args,
                SimulationUiRunnerArgs {
                    max_ticks_per_second: sim_ticks_per_second,
                    max_view_updates_per_second: ui_frame_rate,
                },
            );
        }

        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
}
