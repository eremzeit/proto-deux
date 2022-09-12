use crate::biology::genome::framed::common::*;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::*;
use crate::util::get_from_range;

use rand::Rng;
use std::convert::TryInto;

pub type GenomeAlterationTypeKey = String;
pub type ExecuteGenomeAlterationFn<A> = dyn Fn(&[&CompiledFramedGenome], &[A]) -> Vec<A>;
pub type PrepareAlterationParamsFn<A> = dyn Fn(&[&CompiledFramedGenome]) -> Vec<A>;

pub struct AlterationManifest {
    pub alterations: Vec<GenomeAlterationImplementation>,
}

pub struct AlterationManifestEntry {
    pub key: String,
    pub weight: usize,
}

impl AlterationManifestEntry {
    pub fn new(key: &str, weight: usize) -> Self {
        Self {
            key: key.to_owned(),
            weight,
        }
    }
}

#[derive(Clone)]
pub struct GenomeAlterationImplementation {
    pub key: String,
    pub index: ActionDefinitionIndex,
    pub execute: Rc<ExecuteGenomeAlterationFn<FramedGenomeWord>>,
    pub genomes_required: usize,
    pub prepare: Rc<PrepareAlterationParamsFn<FramedGenomeWord>>,
}

pub struct CompiledAlterationSet {
    pub alterations: Vec<GenomeAlterationImplementation>,
}

impl CompiledAlterationSet {
    pub fn new(alterations: Vec<GenomeAlterationImplementation>) -> CompiledAlterationSet {
        let mut set = CompiledAlterationSet {
            alterations: alterations.clone(),
        };
        set.normalize();
        set
    }

    pub fn from_keys(keys: &Vec<String>) -> Self {
        let all_alterations = default_alterations();
        let invalid = keys
            .iter()
            .filter(|k| all_alterations.iter().find(|a| &a.key == *k).is_none())
            .collect::<Vec<_>>();

        if invalid.len() > 0 {
            panic!("Invalid alteration keys: {:?}", invalid);
        }

        let alterations = default_alterations()
            .into_iter()
            .filter(|a| keys.contains(&a.key.to_string()))
            .to_owned()
            .collect::<Vec<_>>();

        CompiledAlterationSet::new(alterations)
    }

    pub fn alteration_for_key<S: AsRef<str>>(&self, key: S) -> GenomeAlterationImplementation {
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

pub fn default_alteration_set() -> CompiledAlterationSet {
    let mut set = CompiledAlterationSet {
        alterations: default_alterations(),
    };
    set.normalize();
    set
}

pub fn default_alterations() -> Vec<GenomeAlterationImplementation> {
    let mut alterations = vec![];

    alterations.push(GenomeAlterationImplementation {
        key: "insertion".to_string(),
        index: 0,
        genomes_required: 1,
        execute: Rc::new(
            |genomes: &[&CompiledFramedGenome],
             params: &[FramedGenomeWord]|
             -> Vec<FramedGenomeWord> {
                let mut _new = genomes[0].raw_values.iter().map(|x| *x).collect::<Vec<_>>();
                _new.insert(params[0] as usize, params[1] as FramedGenomeWord);
                _new
            },
        ),
        prepare: Rc::new(
            |genomes: &[&CompiledFramedGenome]| -> Vec<FramedGenomeWord> {
                let mut rng = rand::thread_rng();
                vec![
                    rng.gen_range(0..genomes[0].raw_values.len())
                        .try_into()
                        .unwrap(),
                    get_random_genome_word(),
                ]
            },
        ),
    });

    alterations.push(GenomeAlterationImplementation {
        key: "deletion".to_string(),
        index: 0,
        genomes_required: 1,
        execute: Rc::new(
            |genomes: &[&CompiledFramedGenome],
             params: &[FramedGenomeWord]|
             -> Vec<FramedGenomeWord> {
                let mut _new = genomes[0].raw_values.iter().map(|x| *x).collect::<Vec<_>>();
                _new.remove(params[0] as usize);
                _new
            },
        ),
        prepare: Rc::new(
            |genomes: &[&CompiledFramedGenome]| -> Vec<FramedGenomeWord> {
                let mut rng = rand::thread_rng();
                vec![rng
                    .gen_range(0..genomes[0].raw_values.len())
                    .try_into()
                    .unwrap()]
            },
        ),
    });

    alterations.push(GenomeAlterationImplementation {
        key: "random_region_insert".to_string(),
        index: 0,
        genomes_required: 1,
        execute: Rc::new(
            |genomes: &[&CompiledFramedGenome],
             params: &[FramedGenomeWord]|
             -> Vec<FramedGenomeWord> {
                let mut params = params.iter().map(|x| *x).collect::<Vec<_>>();
                let dest_start = params.remove(0);
                let dest_end = params.remove(0);
                let mut region = params;
                // // println!("sectionto splice in {:?}", section);
                // println!("{}, {}", dest_start, dest_end);

                let mut new = genomes[0].clone().raw_values;
                new.splice(dest_start as usize..dest_end as usize, region.into_iter());
                new
            },
        ),
        prepare: Rc::new(
            |genomes: &[&CompiledFramedGenome]| -> Vec<FramedGenomeWord> {
                let mut rng = rand::thread_rng();

                let dest_start = rng.gen_range(0..genomes[0].raw_values.len());
                let mut dest_end = rng.gen_range(dest_start..genomes[0].raw_values.len());

                let region_size = rng.gen_range(0..10);
                let mut params = (0..region_size)
                    .map(|i| get_random_genome_word())
                    .collect::<Vec<_>>();

                params.insert(0, dest_start as FramedGenomeWord);
                params.insert(1, dest_end as FramedGenomeWord);
                params
            },
        ),
    });

    alterations.push(GenomeAlterationImplementation {
        key: "crossover".to_string(),
        index: 0,
        genomes_required: 2,
        execute: Rc::new(
            |genomes: &[&CompiledFramedGenome],
             params: &[FramedGenomeWord]|
             -> Vec<FramedGenomeWord> {
                let src_start = params[0];
                let src_end = params[1]; // exclusive
                let dest_start = params[2];
                let dest_end = params[3]; // exclusive

                let mut section: Vec<FramedGenomeWord> = vec![];

                for i in src_start..src_end {
                    section.push(genomes[0].raw_values[i as usize]);
                }

                // // println!("sectionto splice in {:?}", section);
                // println!("{}, {}", dest_start, dest_end);

                let mut new = genomes[1].clone().raw_values;
                new.splice(
                    dest_start as usize..dest_end as usize,
                    section.clone().into_iter(),
                );
                new
            },
        ),
        prepare: Rc::new(
            |genomes: &[&CompiledFramedGenome]| -> Vec<FramedGenomeWord> {
                let mut rng = rand::thread_rng();

                let src_start = rng.gen_range(0..genomes[0].raw_values.len());
                let mut src_end = rng.gen_range(src_start..genomes[0].raw_values.len());
                src_end = src_end.min(src_start + 50); // TEMP: limit the size of cutout regions as a hack to contain genome sizes

                let dest_start = rng.gen_range(0..genomes[1].raw_values.len());
                let dest_end = rng.gen_range(dest_start..genomes[1].raw_values.len());
                vec![
                    src_start as FramedGenomeWord,
                    src_end as FramedGenomeWord,
                    dest_start as FramedGenomeWord,
                    dest_end as FramedGenomeWord,
                ]
            },
        ),
    });

    alterations.push(GenomeAlterationImplementation {
        key: "point_mutation".to_string(),
        index: 0,
        genomes_required: 1,
        execute: Rc::new(
            |genomes: &[&CompiledFramedGenome],
             params: &[FramedGenomeWord]|
             -> Vec<FramedGenomeWord> {
                let mut _new = genomes[0]
                    .raw_values
                    .iter()
                    .map(|x| *x)
                    .collect::<Vec<u64>>();
                _new[params[0] as usize] = params[1];
                _new
            },
        ),
        prepare: Rc::new(
            |genomes: &[&CompiledFramedGenome]| -> Vec<FramedGenomeWord> {
                let mut rng = rand::thread_rng();
                vec![
                    rng.gen_range(0..genomes[0].raw_values.len())
                        .try_into()
                        .unwrap(),
                    get_random_genome_word(),
                ]
            },
        ),
    });
    alterations.push(GenomeAlterationImplementation {
        key: "point_mutation_in_channel".to_string(),
        index: 0,
        genomes_required: 1,
        execute: Rc::new(
            |genomes: &[&CompiledFramedGenome],
             params: &[FramedGenomeWord]|
             -> Vec<FramedGenomeWord> {
                let idx = params[0] as usize;
                let channel = params[1];
                let val = params[2];

                let mut _new = genomes[0].clone();

                _new.raw_values[idx] = merge_value_into_word(
                    _new.raw_values[idx],
                    val as FramedGenomeValue,
                    channel as u8,
                );

                _new.raw_values
            },
        ),
        prepare: Rc::new(
            |genomes: &[&CompiledFramedGenome]| -> Vec<FramedGenomeWord> {
                let mut rng = rand::thread_rng();
                vec![
                    rng.gen_range(0..genomes[0].raw_values.len())
                        .try_into()
                        .unwrap(),
                    rng.gen_range(0..NUM_CHANNELS).try_into().unwrap(),
                    get_random_genome_value() as FramedGenomeWord,
                ]
            },
        ),
    });

    alterations.push(GenomeAlterationImplementation {
        key: "swap_frames".to_string(),
        index: 0,
        genomes_required: 1,
        execute: Rc::new(
            |genomes: &[&CompiledFramedGenome],
             params: &[FramedGenomeWord]|
             -> Vec<FramedGenomeWord> {
                let frame1_addr = genomes[0].frames[params[0] as usize].address_range;
                let frame2_addr = genomes[0].frames[params[1] as usize].address_range;

                let (earlier_frame, later_frame) = if frame1_addr.0 < frame2_addr.0 {
                    (frame1_addr, frame2_addr)
                } else {
                    (frame2_addr, frame1_addr)
                };

                let mut genome = genomes[0].raw_values.clone();

                let earlier_vals = get_from_range(&genome, earlier_frame);
                let later_vals = get_from_range(&genome, later_frame);

                genome.splice((later_frame.0..later_frame.1), earlier_vals);
                genome.splice((earlier_frame.0..earlier_frame.1), later_vals);

                genome
            },
        ),
        prepare: Rc::new(
            |genomes: &[&CompiledFramedGenome]| -> Vec<FramedGenomeWord> {
                let mut rng = rand::thread_rng();
                vec![
                    rng.gen_range(0..genomes[0].frames.len())
                        .try_into()
                        .unwrap(),
                    rng.gen_range(0..genomes[0].frames.len())
                        .try_into()
                        .unwrap(),
                ]
            },
        ),
    });
    alterations
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
pub mod tests {
    use crate::biology::genome::framed::builders::*;
    use crate::biology::genome::framed::common::FramedGenome;
    use crate::biology::genome::framed::{
        builders::{frame, framed_genome},
        common::{CompiledFramedGenome, FramedGenomeWord},
    };
    use crate::biology::genome::framed_v2::FramedGenomeWithContext;
    use crate::simulation::common::properties::CheeseChemistry;
    use crate::simulation::common::GeneticManifest;

    use super::{default_alterations, CompiledAlterationSet};

    pub fn get_alterations() -> CompiledAlterationSet {
        CompiledAlterationSet::new(default_alterations())
    }

    fn fake_compiled(raw_values: Vec<FramedGenomeWord>) -> CompiledFramedGenome {
        CompiledFramedGenome {
            raw_size: raw_values.len(),
            raw_values,
            frames: vec![],
        }
    }

    #[test]
    pub fn test_insertion() {
        let alteration = get_alterations().alteration_for_key("insertion");
        let genome1 = fake_compiled(vec![1, 2, 3, 4, 5]);

        let genomes = vec![&genome1];
        let params = vec![3, 1337];
        let result = (alteration.execute)(&genomes, &params);

        assert_eq!(result.to_vec(), vec![1, 2, 3, 1337, 4, 5]);
    }

    #[test]
    pub fn test_point_mutation() {
        let alteration = get_alterations().alteration_for_key("point_mutation");
        let genome1 = fake_compiled(vec![1, 2, 3, 4, 5]);

        let genomes = vec![&genome1];
        let params = vec![3, 1337];
        let result = (alteration.execute)(&genomes, &params);

        assert_eq!(result.to_vec(), vec![1, 2, 3, 1337, 5]);
    }

    #[test]
    pub fn test_crossover() {
        let alteration = get_alterations().alteration_for_key("crossover");
        let genome1 = fake_compiled(vec![1, 2, 3, 4, 5]);
        let genome2 = fake_compiled(vec![9, 8, 7]);

        let genomes = vec![&genome1, &genome2];
        let params = vec![1, 3, 1, 2];
        let result = (alteration.execute)(&genomes, &params);

        assert_eq!(result.to_vec(), vec![9, 2, 3, 7]);

        let params = vec![0, 5, 1, 1];
        let result = (alteration.execute)(&genomes, &params);

        println!("result: {:?}", result);
        assert_eq!(result.to_vec(), vec![9, 1, 2, 3, 4, 5, 8, 7]);
    }

    #[test]
    pub fn test_swap_frames() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let alteration = get_alterations().alteration_for_key("swap_frames");

        let genome = framed_genome(vec![
            frame(
                vec![gene(
                    if_any(vec![if_all(vec![conditional!(is_truthy, random_hundred)])]),
                    then_do!(move_unit, up),
                )],
                vec![],
                vec![],
                vec![],
            ),
            frame(
                vec![gene(
                    if_none(vec![if_not_all(vec![conditional!(
                        lt,
                        pos_res::cheese(0, 0),
                        100
                    )])]),
                    then_do!(gobble_cheese, register(3), 69, 69),
                )],
                vec![],
                vec![],
                vec![],
            ),
            frame(
                vec![gene(
                    if_none(vec![if_not_all(vec![conditional!(lt, random(1337), 100)])]),
                    then_do!(new_unit, register(3), 69, 69),
                )],
                vec![],
                vec![],
                vec![],
            ),
        ])
        .build(&gm);

        let genome = FramedGenomeCompiler::compile(genome, &gm);

        let genomes = vec![&genome];
        let params = vec![0, 2];
        let result = (alteration.execute)(&genomes, &params);

        let new_genome = FramedGenomeCompiler::compile(result, &gm);

        let reaction_id = &new_genome.frames[0].channels[0][0]
            .operation
            .clone()
            .as_reaction_call()
            .0;
        let f0_g1_reaction_key = &gm.chemistry_manifest.reactions[*reaction_id as usize].key;

        let reaction_id = &new_genome.frames[2].channels[0][0]
            .operation
            .clone()
            .as_reaction_call()
            .0;
        let f2_g1_reaction_key = &gm.chemistry_manifest.reactions[*reaction_id as usize].key;

        assert_eq!(f0_g1_reaction_key, "new_unit");
        assert_eq!(f2_g1_reaction_key, "move_unit");
    }

    pub fn test_point_mutation_in_channel() {
        let alteration = get_alterations().alteration_for_key("point_mutation_in_channel");
        let genome1 = fake_compiled(vec![1, 2, 0x123, 4, 5]);
        let genomes = vec![&genome1];
        let params = vec![3, 0, 100];
        let result = (alteration.execute)(&genomes, &params);
        assert_eq!(result.to_vec(), vec![1, 2, 3, 0x123, 5]);

        let params = vec![3, 1, 0xaaa];

        let result = (alteration.execute)(&genomes, &params);
        assert_eq!(result.to_vec(), vec![1, 2, 3, 0x0aaa0123, 5]);
    }
}
