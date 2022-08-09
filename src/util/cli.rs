use crate::runners::{RunMode, SimulationRunnerArgs, SimulationUiRunnerArgs};
use clap::{Arg, ArgAction, Command};

pub fn parse_cli_args() -> RunMode {
    let matches = Command::new("proto-molecule")
        .about("A framework for evolving 2d agents")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("sim").about("Run a single simulation").arg(
                Arg::new("sim_scenario_key")
                    .short('s')
                    .long("scenario")
                    .help("A key that selects a predefined simulation configuration (required)")
                    .action(ArgAction::Set)
                    .number_of_values(1),
            ),
        )
        .subcommand(
            Command::new("sim_gui")
                .about("Run a single simulation")
                .arg(
                    Arg::new("sim_scenario_key")
                        .short('s')
                        .long("scenario")
                        .help("A key that selects a predefined simulation configuration (required)")
                        .action(ArgAction::Set)
                        .number_of_values(1),
                )
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
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("sim", sim_matches)) => {
            let sim_scenario_key = sim_matches
                .get_one::<String>("sim_scenario_key")
                .expect("Scenario key required");

            let args = SimulationRunnerArgs {
                simulation_scenario_key: sim_scenario_key.clone(),
                unit_entry_scenario_key: None,
            };

            return RunMode::HeadlessSimulation(args);
        }
        Some(("sim_gui", sim_matches)) => {
            println!("sim matches: {:?}", sim_matches);

            let sim_scenario_key = sim_matches
                .get_one::<String>("sim_scenario_key")
                .expect("Scenario key required");

            let mut sim_ticks_per_second = sim_matches.get_one::<u32>("sim_ticks_per_second");
            let mut ui_frame_rate = sim_matches.get_one::<u32>("ui_frame_rate");

            let args = SimulationRunnerArgs {
                simulation_scenario_key: sim_scenario_key.clone(),
                unit_entry_scenario_key: None,
            };

            return RunMode::GuiSimulation(
                args,
                SimulationUiRunnerArgs {
                    max_ticks_per_second: sim_ticks_per_second.map(|i| *i),
                    max_view_updates_per_second: ui_frame_rate.map(|i| *i),
                },
            );
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
}
