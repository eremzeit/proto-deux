pub mod cheese;
pub mod lever;

pub fn run_one_off(scenario_key: &str) {
    match scenario_key {
        "one_off_lever" => {
            lever::test_fitness(scenario_key);
        }
        "one_off_cheese" => {
            cheese::test_fitness(scenario_key);
        }
        _ => {
            panic!("Scenario key not found: {}", scenario_key);
        }
    };
}
