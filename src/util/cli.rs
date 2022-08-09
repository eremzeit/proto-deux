use crate::runners::{RunMode, SimulationRunnerArgs};
use clap::{Arg, ArgAction, Command};

pub fn parse_cli_args() -> RunMode {
    let matches = Command::new("proto-molecule")
        .about("A framework for evolving 2d agents")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("sim")
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
                    Arg::new("gui")
                        .short('g')
                        .long("gui")
                        .help("Use the gui")
                        .conflicts_with("headless")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("headless")
                        .long("headless")
                        .help("Run without a gui in console only")
                        .conflicts_with("gui")
                        .action(ArgAction::SetTrue),
                ),
        )
        .get_matches();

    println!("args {:?}", matches);

    match matches.subcommand() {
        Some(("sim", sim_matches)) => {
            println!("sim matches: {:?}", sim_matches);

            let sim_scenario_key = sim_matches
                .get_one::<String>("sim_scenario_key")
                .expect("Scenario key required");

            let mut gui = *sim_matches
                .get_one::<bool>("gui")
                .expect("defaulted by clap");

            let mut headless = *sim_matches
                .get_one::<bool>("headless")
                .expect("defaulted by clap");

            // default to gui
            if !gui && !headless {
                gui = true;
                headless = false;
            }

            let args = SimulationRunnerArgs {
                simulation_scenario_key: sim_scenario_key.clone(),
                unit_entry_scenario_key: None,
            };
            return if gui {
                RunMode::GuiSimulation(args)
            } else {
                RunMode::HeadlessSimulation(args)
            };
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
}
