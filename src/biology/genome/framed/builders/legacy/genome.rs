const ENABLE_LOGGING: bool = true;

#[macro_export]
macro_rules! frames_from_genome {
    (($($gene:tt),*)) => ({ 
        genome!($($gene),*)
    })
}

#[macro_export]
macro_rules! genome {
    ($(gene(if_any$if_any:tt, then_do$then_do:tt)),*) => ({ 
        {
            use crate::biology::genome::framed::builders::legacy::util::{GenomeBuilderLegacy, GenomeBuildFunction};
            use crate::biology::genome::framed::types::{FramedGenomeValue};
            use crate::simulation::common::{SensorManifest, GeneticManifest, ChemistryManifest};
            use std::rc::Rc;


            //println!("Building genome...");

            let build_fn: GenomeBuildFunction = Rc::new(
                |genetic_manifest: &GeneticManifest| -> Vec<FramedGenomeValue> {
                    let mut values = vec![];

                    $(
                        //println!("IF_ANY: {}", stringify!($if_any));
                        //println!("THEN_DO: {}", stringify!($then_do));
                        let mut gene_vals = __gene!(genetic_manifest, $if_any, $then_do);
                        values.append(&mut gene_vals);
                    )*

                    values
            });
            GenomeBuilderLegacy::new(build_fn)
        }
    });
}

#[macro_export]
macro_rules! __gene {
    ($gm:expr, 
        $if_any:tt,
        $operation:tt
    ) => ({
        {
            let _gm: &GeneticManifest = $gm;
            let mut v1 = __gene_if_any!(_gm, $if_any);
            let mut v2 = __then_do!(_gm, $operation);

            v1.append(&mut v2);

            v1
        }
    });

}

/*
    EX: sm, gm, (all(...), all(...), ...)
*/
#[macro_export]
macro_rules! __gene_if_any {
    ($gm:expr, ($(all$all_item:tt),*)) => ({
        {
            use crate::biology::genome::framed::types::{FramedGenomeValue};
            let mut size = 0;
            let mut values__any: Vec<FramedGenomeValue> = vec![
                $({
                    let all_items = __gene__all!($gm, $all_item);

                    size += 1;
                    all_items
                }),*
            ].iter().flatten().map(|x| {*x}).collect::<Vec<FramedGenomeValue>>();

            // insert meta value for NON_NEGATED
            values__any.insert(0, 0);
            
            // insert the size of the disjunctive clause
            values__any.insert(0, size);

            values__any
        }
    });
}

/*
    EX: sm, gm, ((eq, 1, 1), (ne, 6, 2))
*/
#[macro_export]
macro_rules! __gene__all{
    ($gm:expr, ($($cond_item:tt),*)) => ({
        {
            use crate::biology::genome::framed::types::{FramedGenomeValue, FIXED_NUM_CONDITIONAL_PARAMS};

            let mut size = 0;
            let mut v: Vec<FramedGenomeValue> = vec![
                $({
                    // the above macro assumes this constant is equal to three so 
                    // we do a sanity check
                    assert_eq!(FIXED_NUM_CONDITIONAL_PARAMS, 3);
                    //println!("inside __gene_all {}", stringify!($cond_item));

                    let bool_var_items = __gene_bool_var!($gm, $cond_item);
                    size += 1;
                    bool_var_items
                }),*
            ].iter().flatten().map(|x| {*x}).collect::<Vec<FramedGenomeValue>>();
            
            // insert meta value for NON_NEGATED
            v.insert(0, 0);

            // insert the size of the disjunctive clause
            v.insert(0, size);

            v
        }
    });
}


// EX: sm, cm, gm, (eq, 1, 1)
#[macro_export]
macro_rules! __gene_bool_var {
    ($gm:expr, ($op_key:expr)) => ({
        __gene_bool_var($gm, ($op_key, ParsedGenomeParam::Constant(200), ParsedGenomeParam::Constant(200), ParsedGenomeParam::Constant(200)))
    });

    ($gm:expr, ($op_key:expr, $param1:expr)) => ({
        __gene_bool_var($gm, ($op_key, $param1, ParsedGenomeParam::Constant(200), ParsedGenomeParam::Constant(200)))
    });
    ($gm:expr, ($op_key:expr, $param1:expr, $param2:expr)) => ({
        __gene_bool_var($gm, ($op_key, $param1, $param2, ParsedGenomeParam::Constant(200)))
    });

    ($gm:expr, ($op_key:expr, $param1:expr, $param2:expr, $param3:expr)) => ({
        {
            use crate::biology::genome::framed::types::{BooleanVariable, FramedGenomeValue};
            use crate::biology::unit_behavior::framed::{ParsedGenomeParam};

            let v: Vec<BooleanVariable> = vec![];
            let op_key = stringify!($op_key).to_string();
            let op = $gm.operator_manifest.by_key(&op_key);

            use crate::biology::genome::framed::convert;
            use crate::biology::genome::framed::convert::param_meta;
            use crate::biology::genome::framed::builders::legacy::util::{GenomeBuilderLegacy, GenomeBuildFunction};
            use crate::biology::genome::framed::util;
            use crate::biology::genetic_manifest::predicates::{OperatorParam};
            use std::convert::TryInto;

            let mut params_meta: [FramedGenomeValue;3] = [0; 3];
            let mut params: [OperatorParam;3] = [0; 3];


            let parsed_operator_param = util::identify_raw_param_string(&stringify!($param1).to_string(), $gm);
            println!("param: {:?}", &parsed_operator_param);
            let p = parsed_operator_param.as_values();
            params_meta[0] = p.0;
            params[0] = p.1.try_into().unwrap();
            
            let parsed_operator_param = util::identify_raw_param_string(&stringify!($param2).to_string(), $gm);
            println!("param: {:?}", &parsed_operator_param);
            let p = parsed_operator_param.as_values();
            params_meta[1] = p.0;
            params[1] = p.1.try_into().unwrap();
            
            let parsed_operator_param = util::identify_raw_param_string(&stringify!($param3).to_string(), $gm);
            println!("param: {:?}", &parsed_operator_param);
            let p = parsed_operator_param.as_values();
            params_meta[2] = p.0;
            params[2] = p.1.try_into().unwrap();

            let is_negated = 0;
            let v: Vec<FramedGenomeValue> = vec![
                op.index as FramedGenomeValue,
                is_negated,
                params_meta[0] as FramedGenomeValue,
                params[0] as FramedGenomeValue,
                params_meta[1] as FramedGenomeValue,
                params[1] as FramedGenomeValue,
                params_meta[2] as FramedGenomeValue,
                params[2] as FramedGenomeValue
            ];

            v
        }
    });
}

macro_rules! __then_do {
    ($gm:expr, ($reaction_name:ident(  $($param:expr),*))) => ({
        {
            use crate::biology::genome::framed::convert::operation;
            use crate::biology::genome::framed::types::{BooleanVariable, FramedGenomeValue, FIXED_NUM_OPERATION_PARAMS};
            use crate::biology::genome::framed::util;
            use crate::biology::genetic_manifest::predicates::{OperatorParam};
            use std::convert::TryInto;
            use crate::biology::unit_behavior::framed::{ParsedGenomeParam, MetaReaction};

            let mut operation_id = None;
            let mut operation_type = None;                

            let reaction_key = stringify!($reaction_name).to_string();

            let meta_reaction = MetaReaction::from_key(&reaction_key);
            println!("meta_reaction: {:?}", &meta_reaction);
            if meta_reaction.is_some() {
                operation_type = Some(operation::val_for_metareaction_operation_type());
                operation_id = Some(meta_reaction.unwrap().to_val());
            }
            
            let reaction = $gm.chemistry_manifest.identify_reaction(&reaction_key);
            if reaction.is_some() {
                let _reaction = reaction.unwrap();
                operation_type = Some(operation::val_for_reaction_operation_type());
                operation_id = Some(_reaction.id as u16);
            }

            if operation_type.is_none() || operation_id.is_none() {
                panic!("Couldnt find op type or op id for key: {:?}", &reaction_key);
            }

            //op type
            //op_id
            //param1_meta
            //param1
            //param2_meta
            //param2
            //param3_meta
            //param3
            let mut v: Vec<FramedGenomeValue> = vec![
                operation_type.unwrap(),
                operation_id.unwrap(),
            ];

            let mut raw_params  = vec![
                $(stringify!($param).to_string()),*
            ];

            let mut params = raw_params.iter().map(|x| {
                util::identify_raw_param_string(x, $gm)
            }).collect::<Vec<ParsedGenomeParam>>();

            while params.len() < FIXED_NUM_OPERATION_PARAMS {
                params.push(ParsedGenomeParam::Constant(0));
            }

            for param in &params {
                let p = param.as_values();
                let param_meta = p.0; 
                let param_value = p.1; 
                v.push(param_meta as FramedGenomeValue);
                v.push(param_value as FramedGenomeValue);
            }

            println!("compled then_do: {:?}", &v);
        
            v
        }

    });
}

#[cfg(test)]
pub mod tests {
    use std::rc::Rc; 
    use crate::simulation::common::variants::cheese::CheeseChemistry;
    use crate::simulation::common::{*};
    use crate::biology::genome::framed::common::{*};
    //use super::super::super::types::{*};

    // use super::super::super::parsing::FramedGenomeParser;
    // use super::super::super::render::{render_frames};
    // use super::super::super::convert::{simple_convert_into_frames};

    #[test]
    fn test_macro__gene_bool_var() {
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let result = __gene_bool_var!(&gm, (is_truthy,1,2,3));

        let op_id = gm.operator_manifest.by_key("is_truthy").index as FramedGenomeValue;
        assert_eq!(result, vec![op_id, 0, 0, 1, 0, 2, 0, 3]);
        //println!("{:?}", &result);
    }

    #[test]
    fn test_macro__then_do__multiple_params() {
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let result1 = __then_do!(&gm, (new_unit(0)));
        let result2 = __then_do!(&gm, (new_unit(0,0)));
        let result3 = __then_do!(&gm, (new_unit(0,0,0)));

        assert_eq!(&result1, &result2);
        assert_eq!(&result2, &result3);
        assert_eq!(&result1, &result3);
    }

    #[test]
    fn test_macro__gene_conjunctive() {
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let result = __gene__all!(&gm, (
            (is_truthy,3,4,5),
            (gte,7,8,9)
        ));

        //println!("{:?}", &result);
        assert_eq!(result, vec![
            2,  // num clauses
            0, // is negated
                1,  // op_id
                0, // is_negated
                0, 3, 0, 4, 0, 5,

                4, 
                0, 
                0, 7, 0, 8, 0, 9]);
    }

    #[test]
    fn test_macro__gene_disjunctive__simple() {
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let result = __gene_if_any!(&gm, (
            all(
                (eq,7,7,7)
            )
        ));

        //println!("{:?}", &result);
        assert_eq!(result, vec![
            1, // num disjunction (OR) clauses
            0, // is negated
                1, // num conjunction (AND) clauses
                0,  // is_negated
                    0, // op_id
                    0,  // is_negated
                    0, 7, 0, 7, 0, 7 // params
        ]);
    }
    #[test]
    fn test_macro__gene_disjunctive() {
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let result = __gene_if_any!(&gm, (
            all(
                (is_truthy,7,7,7),
                (gte,1,2,3),
                (lte,4,5,6)
            ),
            all(
                (is_truthy,1,2,3),
                (gte,9,9,9)
            )
        ));


        //println!("{:?}", &result);
        assert_eq!(result, vec![
            2, // num disjunction (OR) clauses
            0, // is negated
                3, // num conjunction (AND) clauses
                0,  // is_negated
                
                gm.operator_id_for_key("is_truthy") as u16, 
                0, // is_negated
                0, 7, 0, 7, 0, 7,
                
                gm.operator_id_for_key("gte") as u16, 
                0, // is_negated
                0, 1, 0, 2, 0, 3,
                
                gm.operator_id_for_key("lte") as u16, 
                0, // is_negated
                0, 4, 0, 5, 0, 6,

                2, // num conjunction (AND) clauses
                0,  // is_negated
                
                gm.operator_id_for_key("is_truthy") as u16, 
                0, // is_negated
                0, 1, 0, 2, 0, 3,
                
                gm.operator_id_for_key("gte") as u16, 
                0, // is_negated
                0, 9, 0, 9, 0, 9,
                
                
                ]);
    }
    
    #[test]
    fn test_macro__gene_() {
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let result = __gene!(&gm,
            (
                all(
                    (is_truthy,7,7,7),
                    (gte,1,2,3),
                    (lte,4,5,6)
                ),
                all(
                    (is_truthy,0,0,0),
                    (gte,9,9,9)
                )
            ),

            (new_unit(0,0,0))
        );

        println!("{:?}", &result);
    }

    #[test]
    fn test_macro__basic_genome() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let builder = genome!(
                gene(
                    if_any(
                        all(
                            (eq, unit_res::cheese, 1, 2)
                        )
                    ),

                    then_do(new_unit(constant(0)))
                )
        );

        let raw_genome_vals = builder.build(&gm);
        let framed_vals = simple_convert_into_frames(raw_genome_vals);
        let frames = FramedGenomeCompiler::compile(framed_vals, &gm);

        let target = "***FRAME 0:***
Channel #0 (DEFAULT)
CALL new_unit(Constant(0)) IF unit_res::cheese(0, 0) == Constant(1)

Channel #1
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE

Channel #2
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE

Channel #3
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE\n\n";

        println!("{}", frames.display(&gm));
        assert_eq!(
            frames.display(&gm),
            target
        );
    }

    //#[test]
    //fn test_macro__compile_then_parse() {
    //    let gm = GeneticManifest::new();
    //    let cm  = CheeseChemistry::default_manifest();
    //    let sm = SensorManifest::with_default_sensors(&cm);

    //    //trace_macros!(true);
    //    let builder = genome!(
    //            gene(
    //                if_any(
    //                    all(
    //                        (eq, unit_res::cheese, 1, 2)
    //                    ),
    //                    all(
    //                        (eq, unit_res::cheese, 1, 2)
    //                    )
    //                ),

    //                then_do(new_unit(0,0,0))
    //            )
    //    );
    //    //trace_macros!(false);

    //    let genome_values = builder.build(&sm, &cm, &gm);
    //    println!("genome_values: {:?}", &genome_values);

    //    let framed_vals = simple_convert_into_frames(genome_values);
    //    println!("framed genome_values: {:?}", &framed_vals);
    //    let frames = FramedGenomeParser::parse(framed_vals, sm.clone(), gm.clone());

    //    println!("frames: {:?}", &frames);
    //    use super::super::super::render::{render_frames};
    //    let result = render_frames(&frames, &sm, &cm, &gm);
    //    println!("result: {}", &result);
    //}
    
    #[test]
    fn test_macro__compile_then_parse_complex() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let genome_values = genome!(
                gene(
                    if_any(
                        all(
                            (eq, unit_res::cheese, 5, 0),
                            (gt, pos_res::cheese, 2, 0)
                        )
                    ),

                    then_do(new_unit(0,0,0))
                ),
                gene(
                    if_any(
                        all(
                            (is_truthy, pos_attr::is_cheese_source, 1, 0)
                        )
                    ),

                    then_do(move_unit(0,0,0))
                )
        ).build(&gm);

        let framed_vals = simple_convert_into_frames(genome_values);
        let frames = FramedGenomeCompiler::compile(framed_vals, &gm);

        //println!("frames: {:?}", &frames);
        use crate::biology::genome::framed::common::{render_frames};
        println!("result: \n{}", render_frames(&frames.frames, &gm));

        let s = "***FRAME 0:***
Channel #0 (DEFAULT)
CALL new_unit(Constant(0)) IF (unit_res::cheese(0, 0) == Constant(5) && pos_res::cheese(0, 0) > Constant(2))
CALL move_unit(Constant(0)) IF is_truthy(pos_attr::is_cheese_source(0, 0))

Channel #1
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE

Channel #2
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE

Channel #3
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF FALSE\n\n";

        assert_eq!(
            render_frames(&frames.frames, &gm),
            s
        )
    }
}