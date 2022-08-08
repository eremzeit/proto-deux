use biology::genome::framed::common::*;
use biology::phenotype::framed::common::*;
use rand::Rng;
use simulation::common::*;
use std::convert::TryInto;

pub type ExecuteGenomeAlterationFn<A> = dyn Fn(&[Vec<A>], &[A]) -> Vec<A>;
pub type PrepareAlterationParamsFn<A> = dyn Fn(&[Vec<A>]) -> Vec<A>;

#[derive(Clone)]
pub struct GenomeAlterationDefinition {
	pub key: String,
	pub index: ActionDefinitionIndex,
	pub execute: Rc<ExecuteGenomeAlterationFn<FramedGenomeWord>>,
	pub genomes_required: usize,
	pub prepare: Rc<PrepareAlterationParamsFn<FramedGenomeWord>>,
}

pub struct AlterationTypeSet {
	pub alterations: Vec<GenomeAlterationDefinition>,
}

impl AlterationTypeSet {
	pub fn alteration_for_key<S: AsRef<str>>(&self, key: S) -> GenomeAlterationDefinition {
		let _key = key.as_ref();
		self.alterations
			.iter()
			.find(|x| x.key == _key)
			.unwrap()
			.clone()
	}
	pub fn normalize(&mut self) {
		for i in (0..self.alterations.len()) {
			self.alterations[i].index = i;
		}
	}
}

pub fn get_random_genome_word() -> FramedGenomeWord {
	use rand::Rng;
	let mut rng = rand::thread_rng();
	rng.gen_range(0..FramedGenomeWord::MAX)
}

pub fn get_random_genome_value() -> FramedGenomeValue {
	let mut rng = rand::thread_rng();
	rng.gen_range(0..FramedGenomeValue::MAX)
}

pub fn default_alterations() -> AlterationTypeSet {
	let mut alteration_set = AlterationTypeSet {
		alterations: vec![
			GenomeAlterationDefinition {
				key: "insertion".to_string(),
				index: 0,
				genomes_required: 1,
				execute: Rc::new(
					|genomes: &[Vec<FramedGenomeWord>],
					 params: &[FramedGenomeWord]|
					 -> Vec<FramedGenomeWord> {
						let mut _new = genomes[0].iter().map(|x| *x).collect::<Vec<_>>();
						_new[params[0] as usize] = params[1] as FramedGenomeWord;
						_new
					},
				),
				prepare: Rc::new(
					|genomes: &[Vec<FramedGenomeWord>]| -> Vec<FramedGenomeWord> {
						let mut rng = rand::thread_rng();
						vec![
							rng.gen_range(0..genomes[0].len()).try_into().unwrap(),
							get_random_genome_word(),
						]
					},
				),
			},
			GenomeAlterationDefinition {
				key: "deletion".to_string(),
				index: 0,
				genomes_required: 1,
				execute: Rc::new(
					|genomes: &[Vec<FramedGenomeWord>],
					 params: &[FramedGenomeWord]|
					 -> Vec<FramedGenomeWord> {
						let mut _new = genomes[0].iter().map(|x| *x).collect::<Vec<_>>();
						_new.remove(params[0] as usize);
						_new
					},
				),
				prepare: Rc::new(
					|genomes: &[Vec<FramedGenomeWord>]| -> Vec<FramedGenomeWord> {
						let mut rng = rand::thread_rng();
						vec![rng.gen_range(0..genomes[0].len()).try_into().unwrap()]
					},
				),
			},
			// GenomeAlterationDefinition {
			// 	key: "crossover".to_string(),
			// 	index: 0,
			// 	genomes_required: 1,
			// 	execute: Rc::new(
			// 		|genomes: &[Vec<FramedGenomeWord>], params: &[usize]| -> Vec<FramedGenomeWord> {
			// 			genomes[0].iter().map(|x| *x).collect()
			// 		},
			// 	),
			// 	prepare: Rc::new(|genomes: &[&Vec<usize>]| -> Vec<usize> { vec![] }),
			// },
			GenomeAlterationDefinition {
				key: "point_mutation".to_string(),
				index: 0,
				genomes_required: 1,
				execute: Rc::new(
					|genomes: &[Vec<FramedGenomeWord>],
					 params: &[FramedGenomeWord]|
					 -> Vec<FramedGenomeWord> {
						let mut _new = genomes[0].iter().map(|x| *x).collect::<Vec<u64>>();
						_new[params[0] as usize] = params[1];
						_new
					},
				),
				prepare: Rc::new(
					|genomes: &[Vec<FramedGenomeWord>]| -> Vec<FramedGenomeWord> {
						let mut rng = rand::thread_rng();
						vec![
							rng.gen_range(0..genomes[0].len()).try_into().unwrap(),
							get_random_genome_word(),
						]
					},
				),
			},
			// GenomeAlterationDefinition {
			// 	key: "point_mutation_in_channel".to_string(),
			// 	index: 0,
			// 	genomes_required: 1,
			// 	execute: Rc::new(
			// 		|genomes: &[&[FramedGenomeWord]]| -> Vec<FramedGenomeWord> {
			// 			genomes[0].iter().map(|x| *x).collect()
			// 		},
			// 	),
			// },
			// GenomeAlterationDefinition {
			// 	key: "swap_frames".to_string(),
			// 	index: 0,
			// 	genomes_required: 1,
			// 	execute: Rc::new(
			// 		|genomes: &[&[FramedGenomeWord]]| -> Vec<FramedGenomeWord> {
			// 			genomes[0].iter().map(|x| *x).collect()
			// 		},
			// 	),
			// },
			// GenomeAlterationDefinition {
			// 	key: "frames_crossover".to_string(),
			// 	index: 0,
			// 	genomes_required: 2,
			// 	execute: Rc::new(
			// 		|genomes: &[&[FramedGenomeWord]]| -> Vec<FramedGenomeWord> {
			// 			genomes[0].iter().map(|x| *x).collect()
			// 		},
			// 	),
			// },
		],
	};

	alteration_set.normalize();
	alteration_set
}

// const ALTERATION_TYPE_COUNT: usize = 7;
// pub enum GenomeAlterationType {
// 	Insertion,
// 	Deletion,
// 	Crossover,
// 	PointMutation,
// 	PointMutationInChannel,
// 	SwapFrames,
// 	FramesCrossover,
// }

// pub enum GenomeAlteration {
// 	Insertion(usize, FramedGenomeValue),
// 	Deletion(usize, usize),
// 	Crossover(usize, usize, usize, usize), //g1_loc1, g1_loc2, g2_loc1, g2_loc2
// 	PointMutation(usize),                  //
// 	PointMutationInChannel(usize, usize),  //
// 	SwapFrames(usize, usize),              //
// 	FramesCrossover(usize, usize, usize, usize),
// }
