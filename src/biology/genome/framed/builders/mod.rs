#[macro_use]
pub mod legacy;

use crate::biology::genetic_manifest::predicates::OperatorParam;
use crate::chemistry::properties::AttributeIndex;
use crate::simulation::common::*;
use crate::util::{grid_direction_from_string, grid_direction_to_num};
use std::rc::Rc;

pub use crate::biology::genome::framed::convert::*;
pub use crate::biology::genome::framed::parsing::FramedGenomeParser;
pub use crate::biology::genome::framed::types::*;
pub use crate::biology::genome::framed::util::identify_raw_param_string;

use std::convert::TryInto;

type BuildFunction<T> = Rc<dyn Fn(&SensorManifest, &ChemistryManifest, &GeneticManifest) -> Vec<T>>;

macro_rules! _make_builder_type {
    ($builder_type:ident, $output_type:ident) => {
        #[derive(Clone)]
        pub struct $builder_type {
            pub build_fn: BuildFunction<$output_type>,
        }
        impl $builder_type {
            pub fn new(build_fn: BuildFunction<$output_type>) -> Self {
                Self { build_fn }
            }

            pub fn build(
                &self,
                sensor_manifest: &SensorManifest,
                chemistry_manifest: &ChemistryManifest,
                genetic_manifest: &GeneticManifest,
            ) -> Vec<$output_type> {
                (self.build_fn)(sensor_manifest, chemistry_manifest, genetic_manifest)
            }
        }
    };
}

_make_builder_type!(FrameBuilder, FramedGenomeWord);
_make_builder_type!(GeneBuilder, FramedGenomeValue);
_make_builder_type!(PredicateBuilder, FramedGenomeValue);
_make_builder_type!(OperationBuilder, FramedGenomeValue);
_make_builder_type!(ConjunctiveClauseBuilder, FramedGenomeValue);
_make_builder_type!(ConditionalBuilder, FramedGenomeValue);

pub fn frame(default_channel: u8, genes: Vec<GeneBuilder>) -> FrameBuilder {
    FrameBuilder::new(Rc::new(
        move |sm: &SensorManifest,
              cm: &ChemistryManifest,
              gm: &GeneticManifest|
              -> Vec<FramedGenomeWord> {
            let mut v = genes
                .iter()
                .map(|gene| gene.build(sm, cm, gm))
                .collect::<Vec<Vec<FramedGenomeValue>>>()
                .concat();

            v.insert(0, default_channel as FramedGenomeValue);

            let mut frame_vals = v.iter().map(|x| *x as FramedGenomeWord).collect::<Vec<_>>();
            // insert frame size
            frame_vals.insert(0, frame_vals.len() as FramedGenomeWord);
            frame_vals
        },
    ))
}

pub fn gene(predicate: PredicateBuilder, operation: OperationBuilder) -> GeneBuilder {
    GeneBuilder::new(Rc::new(
        move |sm: &SensorManifest,
              cm: &ChemistryManifest,
              gm: &GeneticManifest|
              -> Vec<FramedGenomeValue> {
            let pred_vals = predicate.build(sm, cm, gm);
            flog!("pred_values: {:?}", &pred_vals);

            let op_vals = operation.build(sm, cm, gm);
            vec![pred_vals, op_vals].concat()
        },
    ))
}

pub fn if_any(conjunctive_clauses: Vec<ConjunctiveClauseBuilder>) -> PredicateBuilder {
    let result = _predicate(conjunctive_clauses, 0);

    result
}

pub fn if_none(conjunctive_clauses: Vec<ConjunctiveClauseBuilder>) -> PredicateBuilder {
    _predicate(conjunctive_clauses, 1)
}

fn _predicate(
    conjunctive_clauses: Vec<ConjunctiveClauseBuilder>,
    is_negated: FramedGenomeValue,
) -> PredicateBuilder {
    PredicateBuilder::new(Rc::new(
        move |sm: &SensorManifest,
              cm: &ChemistryManifest,
              gm: &GeneticManifest|
              -> Vec<FramedGenomeValue> {
            let mut values = conjunctive_clauses
                .iter()
                .map(|clause| clause.build(sm, cm, gm))
                .collect::<Vec<_>>()
                .concat();

            values.insert(0, is_negated);

            values.insert(0, conjunctive_clauses.len() as FramedGenomeValue);

            values
        },
    ))
}

pub fn if_all(conditionals: Vec<ConditionalBuilder>) -> ConjunctiveClauseBuilder {
    _if_all(conditionals, 0)
}

pub fn if_not_all(conditionals: Vec<ConditionalBuilder>) -> ConjunctiveClauseBuilder {
    _if_all(conditionals, 1)
}

fn _if_all(
    conditionals: Vec<ConditionalBuilder>,
    is_negated: FramedGenomeValue,
) -> ConjunctiveClauseBuilder {
    ConjunctiveClauseBuilder::new(Rc::new(
        move |sm: &SensorManifest,
              cm: &ChemistryManifest,
              gm: &GeneticManifest|
              -> Vec<FramedGenomeValue> {
            let mut values = conditionals
                .iter()
                .map(|clause| clause.build(sm, cm, gm))
                .collect::<Vec<_>>()
                .concat();
            values.insert(0, is_negated); // is_negated
            values.insert(0, conditionals.len() as FramedGenomeValue);
            flog!("if_all: {:?}", values);
            values
        },
    ))
}

#[macro_export]
macro_rules! conditional {
    ($op_key:ident ) => {
        conditional!($op_key, 0, 0, 0)
    };
    ($op_key:ident, $param1:expr) => {
        conditional!($op_key, $param1, 0, 0)
    };

    ($op_key:ident, $param1:expr, $param2:expr) => {
        conditional!($op_key, $param1, $param2, 0)
    };

    ($op_key:ident, $param1:expr, $param2:expr, $param3:expr) => {{
        ConditionalBuilder::new(Rc::new(
            |sm: &SensorManifest,
             cm: &ChemistryManifest,
             gm: &GeneticManifest|
             -> Vec<FramedGenomeValue> {
                use crate::biology::genetic_manifest::predicates::OperatorParam;
                use std::convert::TryInto;
                let v: Vec<BooleanVariable> = vec![];
                let op_key = stringify!($op_key).to_string();
                let op = gm.operator_set.by_key(&op_key);

                let mut params_meta: [FramedGenomeValue; 3] = [0; 3];
                let mut params: [OperatorParam; 3] = [0; 3];

                let parsed_operator_param =
                    identify_raw_param_string(&stringify!($param1).to_string(), sm, gm);
                let mut p = parsed_operator_param.as_values();
                params_meta[0] = p.0;
                params[0] = p.1.try_into().unwrap();

                let parsed_operator_param =
                    identify_raw_param_string(&stringify!($param2).to_string(), sm, gm);
                let mut p = parsed_operator_param.as_values();
                params_meta[1] = p.0;
                params[1] = p.1.try_into().unwrap();

                let parsed_operator_param =
                    identify_raw_param_string(&stringify!($param3).to_string(), sm, gm);
                let mut p = parsed_operator_param.as_values();
                params_meta[2] = p.0;
                params[2] = p.1.try_into().unwrap();

                let is_negated = 0;

                let v: Vec<FramedGenomeValue> = vec![
                    op.index as FramedGenomeValue,
                    is_negated as FramedGenomeValue,
                    params_meta[0] as FramedGenomeValue,
                    params[0] as FramedGenomeValue,
                    params_meta[1] as FramedGenomeValue,
                    params[1] as FramedGenomeValue,
                    params_meta[2] as FramedGenomeValue,
                    params[2] as FramedGenomeValue,
                ];

                v
            },
        ))
    }};
}

#[macro_export]
macro_rules! then_do {
    ($op_key:ident) => {
        then_do!($op_key, 0, 0, 0)
    };

    ($op_key:ident, $param1:expr) => {
        then_do!($op_key, $param1, 0, 0)
    };

    ($op_key:ident, $param1:expr, $param2:expr) => {
        then_do!($op_key, $param1, $param2, 0)
    };

    // TODO: extract this below logic into a single function.  The macro should just be syntactic sugar.
    ($op_key:ident, $param1:expr, $param2:expr, $param3:expr) => {{
        {
            use crate::biology::genome::framed::common::*;
            use std::convert::TryInto;

            OperationBuilder::new(Rc::new(
                |sm: &SensorManifest,
                 cm: &ChemistryManifest,
                 gm: &GeneticManifest|
                 -> Vec<FramedGenomeValue> {
                    pub use crate::biology::genetic_manifest::predicates::OperatorParam;
                    use crate::biology::phenotype::framed::MetaReaction;

                    let v: Vec<BooleanVariable> = vec![];
                    let op_key = stringify!($op_key).to_string();

                    let mut operation_id = None;
                    let mut operation_type = None;

                    let operation_key = stringify!($op_key).to_string();

                    // TODO: allow customizing which reactions and meta-reactions are available to this genome.
                    // this means creating an object called something like GenomeOperationManifest, which is generated
                    // as a subset of the chemistry manifest.  It would contain a list of entries that map from the operation manifest
                    // to the metareaction manifest and reaction manifest.

                    let meta_reaction = MetaReaction::from_key(&operation_key);
                    println!("meta_reaction: {:?}", &meta_reaction);
                    if meta_reaction.is_some() {
                        operation_type = Some(operation::val_for_metareaction_operation_type());
                        operation_id = Some(meta_reaction.unwrap().to_val());
                    }


                    let reaction = cm.identify_reaction(&operation_key);
                    if reaction.is_some() {
                        let _reaction = reaction.unwrap();
                        operation_type = Some(operation::val_for_reaction_operation_type());
                        operation_id = Some(_reaction.id as u16);
                    }

                    if operation_type.is_none() || operation_id.is_none() {
                        panic!(
                            "Couldnt find op type or op id for key: {:?}",
                            &operation_key
                        );
                    }

                    let mut params_meta: [FramedGenomeValue; 3] = [0; 3];
                    let mut params: [OperatorParam; 3] = [0; 3];

                    let parsed_operator_param =
                        identify_raw_param_string(&stringify!($param1).to_string(), sm, gm);
                    let mut p = parsed_operator_param.as_values();
                    params_meta[0] = p.0;
                    params[0] = p.1.try_into().unwrap();

                    let parsed_operator_param =
                        identify_raw_param_string(&stringify!($param2).to_string(), sm, gm);
                    let mut p = parsed_operator_param.as_values();
                    params_meta[1] = p.0;
                    params[1] = p.1.try_into().unwrap();

                    let parsed_operator_param =
                        identify_raw_param_string(&stringify!($param3).to_string(), sm, gm);
                    let mut p = parsed_operator_param.as_values();
                    params_meta[2] = p.0;
                    params[2] = p.1.try_into().unwrap();

                    let v: Vec<FramedGenomeValue> = vec![
                        operation_type.unwrap() as FramedGenomeValue,
                        operation_id.unwrap() as FramedGenomeValue,
                        params_meta[0] as FramedGenomeValue,
                        params[0] as FramedGenomeValue,
                        params_meta[1] as FramedGenomeValue,
                        params[1] as FramedGenomeValue,
                        params_meta[2] as FramedGenomeValue,
                        params[2] as FramedGenomeValue,
                    ];

                    v
                },
            ))
        }
    }};
}

macro_rules! flat_vec {
    ($($val:expr),*) => (
        vec![
            $($val),*
        ].iter().flatten().map(|x| {*x as FramedGenomeValue}).collect::<Vec<_>>()
    );
}

macro_rules! _vals_as_conditional {
    ($first_val:expr, $(val:expr),*) => ({

        {
            let vals = vec![
                $(val),*
            ];

            assert_eq!(vals.len(), 8);
        },

        $($val),*
    });
}

pub mod parsing_v2 {
    //use super::convert::param_meta;
    //use super::*;
    use crate::biology::genome::framed::common::*;
    use crate::biology::phenotype::framed::common::*;
    use crate::simulation::common::variants::CheeseChemistry;
    use crate::simulation::common::*;

    #[test]
    fn conditional() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let raw_conditional =
            conditional!(is_truthy, pos_attr::is_cheese_source(0, 0)).build(&sm, &cm, &gm);

        let operator_id = gm.operator_id_for_key("is_truthy") as FramedGenomeValue;
        let is_negated = 0 as FramedGenomeValue;
        let param1_meta = param_meta::val_for_sensor_lookup() as FramedGenomeValue;
        let sensor_id = sm
            .identify_sensor_from_key(&"pos_attr::is_cheese_source(0, 0)".to_string())
            .unwrap()
            .id as FramedGenomeValue;
        assert_eq!(raw_conditional.len(), 8);
        assert_eq!(
            raw_conditional,
            vec![operator_id, is_negated, param1_meta, sensor_id, 0, 0, 0, 0]
        );
    }
    #[test]
    fn if_any_test() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let op_id = gm.operator_id_for_key("is_truthy") as FramedGenomeValue;
        let is_negated = 0 as FramedGenomeValue;
        let param1_meta = param_meta::val_for_sensor_lookup() as FramedGenomeValue;
        let sensor_id = sm
            .identify_sensor_from_key(&"pos_attr::is_cheese_source(0, 0)")
            .unwrap()
            .id as FramedGenomeValue;

        let values = if_any(vec![if_all(vec![conditional!(
            is_truthy,
            pos_attr::is_cheese_source(0, 0)
        )])])
        .build(&sm, &cm, &gm);

        assert_eq!(
            values,
            vec![
                1,
                0,
                1,
                0,
                op_id,
                is_negated,
                param1_meta,
                sensor_id,
                0,
                0,
                0,
                0
            ]
        );

        //let genome = FramedGenomeParser::parse(values, cm.clone(), sm.clone(), gm.clone());
    }

    #[test]
    fn test_then_do__register_param() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let operation_type = 0;
        let reaction_id = cm
            .identify_reaction(&"gobble_cheese".to_string())
            .unwrap()
            .id as FramedGenomeValue;
        let param1_meta = param_meta::val_for_register_lookup() as FramedGenomeValue;
        let result__register = then_do!(gobble_cheese, register(1)).build(&sm, &cm, &gm);

        assert_eq!(
            result__register,
            vec![operation_type, reaction_id, param1_meta, 1, 0, 0, 0, 0]
        );
    }
    // #[test]
    // fn test_as_values_macro() {
    //     assert_eq!(
    //         _vals_as_conditional!(1,2,3,4,5,6,7,8),
    //         vec![1,2,3,4,5,6,7,8],
    //     )

    // }
    #[test]
    fn test_gene() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);
        let gene_values = gene(
            if_any(vec![if_not_all(vec![conditional!(
                is_truthy,
                pos_attr::is_cheese_source(0, 0)
            )])]),
            then_do!(move_unit, register(3), 69, 70),
        )
        .build(&sm, &cm, &gm);

        let n_conjuctions = 1;
        let disjunction1_is_negated = 0;
        let operation_type = 0;
        let reaction_id =
            cm.identify_reaction(&"move_unit".to_string()).unwrap().id as FramedGenomeValue;

        let n_conditionals = 1;
        let conjunction1_is_negated = 1;

        let conditional1_is_negated = 0;
        let reaction_param1_meta = param_meta::val_for_register_lookup() as FramedGenomeValue;

        let target = flat_vec![
            flat_vec![
                vec![n_conjuctions, disjunction1_is_negated],
                flat_vec![
                    vec![n_conditionals, conjunction1_is_negated],
                    vec![
                        gm.operator_id_for_key(&"is_truthy") as FramedGenomeValue,
                        conditional1_is_negated,
                        param_meta::val_for_sensor_lookup() as FramedGenomeValue,
                        sm.sensor_id_from_key(&"pos_attr::is_cheese_source(0, 0)")
                            as FramedGenomeValue,
                        0,
                        0,
                        0,
                        0
                    ]
                ]
            ],
            vec![
                operation_type,
                reaction_id,
                reaction_param1_meta,
                3,
                0,
                69,
                0,
                70
            ]
        ];
        assert_eq!(&gene_values, &target);
    }

    #[test]
    fn full_parsing__basic_genome() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let framed_vals = frame(
            0,
            vec![gene(
                if_any(vec![if_all(vec![conditional!(
                    is_truthy,
                    pos_attr::is_cheese_source(0, 0)
                )])]),
                then_do!(move_unit, 75),
            )],
        )
        .build(&sm, &cm, &gm);

        let genome = FramedGenomeParser::parse(framed_vals, cm.clone(), sm.clone(), gm.clone());
        let s = "***FRAME 0:***
Channel #0
CALL move_unit(Constant(75)) IF is_truthy(pos_attr::is_cheese_source(0, 0))

Channel #1

Channel #2

Channel #3\n\n";
        assert_eq!(s, render_frames(&genome.frames, &sm, &cm, &gm))
    }
    #[test]
    fn full_parsing__complex_genome() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        println!("BEFORE COMPILING");
        let framed_vals = frame(
            0,
            vec![
                gene(
                    if_any(vec![if_all(vec![
                        conditional!(is_truthy, pos_attr::is_cheese_source(0, 0)),
                        conditional!(gt, unit_res::cheese, 100),
                    ])]),
                    then_do!(move_unit, 75),
                ),
                gene(
                    if_none(vec![if_not_all(vec![conditional!(
                        lt,
                        sim_attr::total_cheese_consumed,
                        100
                    )])]),
                    then_do!(new_unit, register(3), 69, 69),
                ),
            ],
        )
        .build(&sm, &cm, &gm);

        let genome = FramedGenomeParser::parse(framed_vals, cm.clone(), sm.clone(), gm.clone());
        let s = "***FRAME 0:***
Channel #0
CALL move_unit(Constant(75)) IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && unit_res::cheese(0, 0) > Constant(100))
CALL new_unit(Register(3)) IF NOT NOT sim_attr::total_cheese_consumed(0, 0) < Constant(100)

Channel #1

Channel #2

Channel #3\n\n";

        println!("{}", &s);
        println!("{}", &render_frames(&genome.frames, &sm, &cm, &gm));

        assert_eq!(s, render_frames(&genome.frames, &sm, &cm, &gm));
    }
}
