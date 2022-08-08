use simulation::common::*;
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
			key: "total_cheese_consumed".to_string(),
			execute: Rc::new(|unit_entry_id: usize, sim: &SimCell| -> FitnessScore {
				let manifest = sim.chemistry.get_manifest();
				let attr_id = manifest
					.unit_entry_attribute_by_key("total_cheese_consumed")
					.id;
				let attr_val = &sim.unit_entry_attributes[unit_entry_id][attr_id]; // as FitnessScore
				attr_val.coerce_unwrap_to_integer() as FitnessScore
			}),
		},
	]
}

pub fn calculate_fitness(
	// fitnessDef: &FitnessCalculationDefinition,
	fitnessDefKey: &String,
	unit_entry_id: usize,
	sim: &SimCell,
) -> FitnessScore {
	let calculators = default_fitness_calculators();
	let fitnessDef = calculators
		.iter()
		.find(|x| &x.key == fitnessDefKey)
		.unwrap();
	(fitnessDef.execute)(unit_entry_id, sim)
}
