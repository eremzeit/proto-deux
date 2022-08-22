pub mod cheese;
pub mod lever;
pub mod perf;

pub fn run_one_off(scenario_key: &str) {
    match scenario_key {
        "one_off_lever" => {
            lever::test_fitness(scenario_key);
        }
        "one_off_cheese" => {
            cheese::test_fitness(scenario_key);
        }
        "test_closure_perf" => {
            perf::closures::test_closure_perf();
        }
        "test_sim_perf" => {
            perf::sim::test_sim_perf();
        }
        _ => {
            panic!("Scenario key not found: {}", scenario_key);
        }
    };
}
