use crate::simulation::common::*;
use std::rc::Rc;

pub type FitnessScore = u64;

pub type CalculateFitnessFn = dyn Fn(usize, &SimCell) -> FitnessScore;

#[derive(Clone)]
pub struct FitnessCalculationDefinition {
    pub key: String,
    pub execute: Rc<CalculateFitnessFn>,
}

pub fn default_fitness_calculators() -> Vec<FitnessCalculationDefinition> {
    vec![
        FitnessCalculationDefinition {
            key: "lever_pulls".to_string(),
            execute: Rc::new(|unit_entry_id: usize, sim: &SimCell| -> FitnessScore {
                let manifest = sim.chemistry.get_manifest();
                let attr_id = manifest.unit_entry_attribute_by_key("lever_pulls").id;
                let attr_val = &sim.unit_entry_attributes[unit_entry_id][attr_id]; // as FitnessScore
                attr_val.coerce_unwrap_to_integer() as FitnessScore
            }),
        },
        FitnessCalculationDefinition {
            key: "total_cheese_acquired".to_string(),
            execute: Rc::new(|unit_entry_id: usize, sim: &SimCell| -> FitnessScore {
                let manifest = sim.chemistry.get_manifest();
                let attr_id = manifest
                    .unit_entry_attribute_by_key("total_cheese_acquired")
                    .id;
                let attr_val = &sim.unit_entry_attributes[unit_entry_id][attr_id]; // as FitnessScore
                attr_val.coerce_unwrap_to_integer() as FitnessScore
            }),
        },
    ]
}

pub fn calculate_fitness(
    // fitnessDef: &FitnessCalculationDefinition,
    fitness_def_key: &String,
    unit_entry_id: usize,
    sim: &SimCell,
) -> FitnessScore {
    let calculators = default_fitness_calculators();
    let fitnessDef = calculators
        .iter()
        .find(|x| &x.key == fitness_def_key)
        .unwrap();
    (fitnessDef.execute)(unit_entry_id, sim)
}

#[cfg(test)]
pub mod tests {
    use crate::{runners::SimulationRunnerArgs, scenarios::simulations::get_simulation_scenario};

    #[test]

    pub fn test_lever_pull_fitness() {
        let sim_args = SimulationRunnerArgs {
            chemistry_key: "lever".to_string(),
            simulation_scenario_key: "test_fitness".to_string(),
            unit_entry_scenario_key: Some("single".to_string()),
            iterations: Some(10),
        };
    }
}
