use crate::simulation::{
    common::{ChemistryManifest, UnitManifest},
    fitness::FitnessScore,
    SimCell,
};

pub struct FitnessCalculator {
    chemistry_manifest: ChemistryManifest,
    unit_manifest: UnitManifest,
}

fn calculate_fitness(sim: &SimCell, fitness_calculator: FitnessCalculator) -> FitnessScore {
    0
}

/*
    thought: Fitness calculation depends on the choice of chemistry (or just chemistry manifest)
    HOWEVER, fitness is not a part of the chemistry itself.
*/

// pub trait SubType {

// }

// pub struct MyStruct<T> {
//     pub attr: usize,
// 	pub sub_type: T,
// }

// impl MyStruct {

// }
