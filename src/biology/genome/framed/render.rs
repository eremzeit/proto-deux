use super::types::*;
use super::types::{BooleanVariable, ConjunctiveClause};
use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorId, OperatorParam, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::sensor_manifest::SensorManifest;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry;
use crate::chemistry::properties::AttributeIndex;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use std::fmt::{Debug, Formatter, Result};

use crate::biology::unit_behavior::framed::{GeneOperationCall, ParsedGenomeParam};

pub fn render_param(param: &ParsedGenomeParam, sensor_manifest: &SensorManifest) -> String {
    match param {
        ParsedGenomeParam::Constant(x) => format!("Constant({})", x),
        ParsedGenomeParam::SensorLookup(sensor_id) => {
            let sensor = &sensor_manifest.sensors[*sensor_id];
            sensor.key.clone()
        }
        ParsedGenomeParam::Register(register_id) => {
            format!("Register({})", register_id)
        }
        ParsedGenomeParam::Random(max_val) => {
            format!("Random({})", max_val)
        }
    }
}

pub fn render_frames(frames: &Vec<Frame>, genetic_manifest: &GeneticManifest) -> String {
    let mut s = String::new();

    for (frame_i, frame) in frames.iter().enumerate() {
        s.push_str(&format!("***FRAME {}:***\n", frame_i));
        for channel in (0..4) {
            let gene_str = render_genes(&frame.channels[channel], genetic_manifest);
            let is_default = frame.default_channel as usize == channel;
            let default_str = if is_default {
                " (DEFAULT)".to_owned()
            } else {
                "".to_owned()
            };
            s.push_str(&format!("Channel #{}{}\n", channel, &default_str));
            s.push_str(&format!("{}\n", &gene_str));
        }
    }

    s
}

pub fn render_genes(genes: &Vec<Gene>, genetic_manifest: &GeneticManifest) -> String {
    let mut s = String::new();

    for gene in genes {
        let gene_str = render_gene(gene, genetic_manifest);
        s.push_str(&format!("{}\n", &gene_str));
    }

    s
}

pub fn render_gene(gene: &Gene, genetic_manifest: &GeneticManifest) -> String {
    let disjunctive_clause = &gene.conditional;
    let gene_op_call = &gene.operation;

    let clause_str = render_disjunction(&disjunctive_clause, genetic_manifest);
    let gene_op_str = render_gene_operation(&gene_op_call, genetic_manifest);

    format!("CALL {} IF {}", gene_op_str, clause_str)
}

pub fn render_conjunction(
    clause: &ConjunctiveClause,
    genetic_manifest: &GeneticManifest,
) -> String {
    let _items = clause.1.iter();
    let is_negated = clause.0;

    let result = _items.fold(
        "".to_string(),
        |acc: String, x: &BooleanVariable| -> String {
            let s = format!("{}", x.render(genetic_manifest));

            if acc.len() == 0 {
                format!("{}", &x.render(genetic_manifest))
            } else {
                format!("({} && {})", acc, &x.render(genetic_manifest))
            }
        },
    );

    let is_negated_str = if is_negated { "NOT " } else { "" }.to_string();
    format!("{}{}", is_negated_str, result)
}

pub fn render_disjunction(dis: &DisjunctiveClause, genetic_manifest: &GeneticManifest) -> String {
    let _items = dis.1.iter();
    let is_negated = dis.0;

    let result = _items.fold(
        "".to_string(),
        |acc: String, x: &ConjunctiveClause| -> String {
            if acc.len() == 0 {
                format!("{}", render_conjunction(x, genetic_manifest))
            } else {
                format!("( {} || {} )", acc, render_conjunction(x, genetic_manifest))
            }
        },
    );

    let is_negated_str = if is_negated { "NOT " } else { "" }.to_string();
    format!("{}{}", is_negated_str, result)
}

pub fn render_gene_operation(
    call: &GeneOperationCall,
    genetic_manifest: &GeneticManifest,
) -> String {
    match &call {
        GeneOperationCall::Reaction((reaction_id, p1, p2, p3)) => {
            let reaction = &genetic_manifest.chemistry_manifest.reactions[*reaction_id as usize];
            let required_count = genetic_manifest
                .chemistry_manifest
                .get_required_params_for_reaction(&reaction.key);

            if required_count == 0 {
                format!("{}()", reaction.key)
            } else if required_count == 1 {
                format!("{}({:?})", reaction.key, p1)
            } else if required_count == 2 {
                format!("{}({:?}, {:?})", reaction.key, p1, p2)
            } else {
                format!("{}({:?}, {:?}, {:?})", reaction.key, p1, p2, p3)
            }
        }
        GeneOperationCall::MetaReaction(meta_reaction) => {
            format!("{:?}", meta_reaction)
        }
        GeneOperationCall::Nil => {
            format!("NilGeneOperationCall")
        }
    }
}

pub mod tests {
    use chemistry::variants::CheeseChemistry;

    use super::*;
    use crate::biology::genetic_manifest::predicates::default_operators;
    use crate::biology::genome::framed::*;
    use crate::biology::unit_behavior::framed::*;
    use crate::simulation::common::*;

    #[test]
    pub fn conjunctive_to_str__simple() {
        let clause = (
            false,
            vec![
                BooleanVariable::Literal(true),
                BooleanVariable::Literal(true),
                BooleanVariable::Literal(true),
            ],
        );

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());

        let result = render_conjunction(&clause, &gm);
        //println!("RESULT: {}", &result);
        assert_eq!(
            result,
            "((Value(true) && Value(true)) && Value(true))".to_string()
        );
    }

    #[test]
    pub fn conjunctive_to_str__conditional1() {
        let clause = (
            false,
            vec![
                BooleanVariable::Conditional(
                    0,
                    false,
                    ParsedGenomeParam::SensorLookup(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::SensorLookup(0),
                ),
                BooleanVariable::Literal(true),
            ],
        );

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());

        let result = render_conjunction(&clause, &gm);

        assert_eq!(
            result,
            "(sim_attr::total_cheese_consumed(0, 0) == Constant(0) && Value(true))".to_string()
        );
    }

    #[test]
    pub fn conjunctive_to_str__conditional2() {
        let clause = (
            false,
            vec![
                BooleanVariable::Conditional(
                    1,
                    false,
                    ParsedGenomeParam::SensorLookup(0),
                    ParsedGenomeParam::Constant(100),
                    ParsedGenomeParam::Constant(10),
                ),
                BooleanVariable::Literal(true),
            ],
        );

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());

        let result = render_conjunction(&clause, &gm);

        assert_eq!(
            result,
            "(is_truthy(sim_attr::total_cheese_consumed(0, 0)) && Value(true))".to_string()
        );
    }

    #[test]
    pub fn disjunctive_to_str__simple() {
        let clause = (false, vec![(false, vec![BooleanVariable::Literal(true)])]);

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());

        let result = render_disjunction(&clause, &gm);

        assert_eq!(result, "Value(true)".to_string());
    }
    #[test]
    pub fn disjunctive_to_str__negated() {
        let clause = (true, vec![(false, vec![BooleanVariable::Literal(true)])]);

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());
        let cm = CheeseChemistry::default_manifest();

        let result = render_disjunction(&clause, &gm);

        assert_eq!(result, "NOT Value(true)".to_string());
    }

    #[test]
    pub fn disjunctive_to_str__complex1() {
        let clause = (
            true,
            vec![
                (false, vec![BooleanVariable::Literal(true)]),
                (
                    true,
                    vec![
                        BooleanVariable::Literal(true),
                        BooleanVariable::Literal(false),
                    ],
                ),
            ],
        );

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());
        let cm = CheeseChemistry::default_manifest();
        let result = render_disjunction(&clause, &gm);

        assert_eq!(
            result,
            "NOT ( Value(true) || NOT (Value(true) && Value(false)) )".to_string()
        );
    }

    #[test]
    pub fn render_gene__simple() {
        let gene = Gene {
            conditional: (
                false,
                vec![
                    (false, vec![BooleanVariable::Literal(true)]),
                    (
                        false,
                        vec![
                            BooleanVariable::Literal(true),
                            BooleanVariable::Literal(false),
                        ],
                    ),
                ],
            ),

            operation: GeneOperationCall::Nil,
        };

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());

        let result = render_gene(&gene, &gm);

        assert_eq!(
            result,
            "CALL NilGeneOperationCall IF ( Value(true) || (Value(true) && Value(false)) )"
                .to_string()
        );
    }

    #[test]
    pub fn render_genes__simple() {
        let genes: Vec<Gene> = vec![
            Gene {
                conditional: (
                    false,
                    vec![
                        (false, vec![BooleanVariable::Literal(false)]),
                        (false, vec![BooleanVariable::Literal(true)]),
                    ],
                ),

                operation: GeneOperationCall::Reaction((
                    0,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            },
            Gene {
                conditional: (
                    false,
                    vec![
                        (false, vec![BooleanVariable::Literal(true)]),
                        (false, vec![BooleanVariable::Literal(false)]),
                    ],
                ),

                operation: GeneOperationCall::Reaction((
                    1,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            },
        ];

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());

        let result = render_genes(&genes, &gm);
        let expected = "CALL gobble_cheese() IF ( Value(false) || Value(true) )
CALL move_unit(Constant(0)) IF ( Value(true) || Value(false) )\n";

        println!("RESULT: \n{}", &result);
        assert_eq!(result, expected);
    }

    #[test]
    pub fn render_genes__multi_channel() {
        let genes: Vec<Gene> = vec![
            Gene {
                conditional: (
                    false,
                    vec![
                        (false, vec![BooleanVariable::Literal(true)]),
                        (false, vec![BooleanVariable::Literal(true)]),
                    ],
                ),

                operation: GeneOperationCall::Reaction((
                    0,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            },
            Gene {
                conditional: (
                    true,
                    vec![
                        (false, vec![BooleanVariable::Literal(true)]),
                        (false, vec![BooleanVariable::Literal(true)]),
                    ],
                ),
                operation: GeneOperationCall::Reaction((
                    1,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            },
        ];

        let gm = GeneticManifest::defaults(&CheeseChemistry::default_manifest());

        let result = render_genes(&genes, &gm);
        let expected = "CALL gobble_cheese() IF ( Value(true) || Value(true) )
CALL move_unit(Constant(0)) IF NOT ( Value(true) || Value(true) )\n";

        assert_eq!(result, expected);
    }
}
