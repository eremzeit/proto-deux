use crate::{
    biology::experiments::variants::simple::SimpleExperiment, runners::ExperimentRunnerArgs,
};

pub mod cheese;
pub mod lever;

pub fn get_experiment_scenario(runner_args: ExperimentRunnerArgs) -> SimpleExperiment {
    match runner_args.experiment_scenario_key.as_str() {
        "simple_lever" => lever::simple_experiment(runner_args.clone()),
        "simple_cheese" => cheese::simple_experiment(runner_args.clone()),
        "multi_pool_cheese" => cheese::multi_pool_cheese_experiment(runner_args.clone()),
        _ => panic!("scenario not defined"),
    }
}
