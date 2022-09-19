use super::super::annotated::{
    ChannelExecutionStats, ConjunctionExpressionStats, DisjunctionExpressionStats,
    FramedGenomeExecutionStats, GeneExecutionStats,
};
use super::super::types::*;
use super::super::types::{BooleanVariable, Conjunction};
use crate::biology::genetic_manifest::predicates::{
    OperatorId, OperatorImplementation, OperatorManifest, OperatorParam, OperatorParamDefinition,
    OperatorParamType,
};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::common::{CompiledFramedGenome, Disjunction, Gene};
use crate::biology::sensor_manifest::SensorManifest;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry;
use crate::chemistry::properties::AttributeIndex;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use std::fmt::{Debug, Formatter, Result};

use crate::biology::unit_behavior::framed::{ParamedGeneOperationCall, ParsedGenomeParam};

// pub struct FramedGenomeRenderer {}

// impl FramedGenomeRenderer {
//     pub fn render(genome: &CompiledFramedGenome) -> String {}
// }

const INDENT_SIZE: usize = 4;

pub fn render_frames_with_stats(
    frames: &Vec<Frame>,
    genetic_manifest: &GeneticManifest,
    mut stats: Option<&FramedGenomeExecutionStats>,
) -> String {
    let mut s = String::new();

    if stats.is_some() && frames.len() != stats.unwrap().frames.len() {
        panic!("Stats hasn't been initialized");
    }

    for (frame_i, frame) in frames.iter().enumerate() {
        let pct_str = if let Some(_stats) = stats {
            display_pct(_stats.frames[frame_i].pct_true())
        } else {
            String::new()
        };

        let frame_header = format!("***FRAME {}:***", frame_i);
        s.push_str(&format!("{} {}", &pct_str, frame_header.as_str()));

        if stats.is_some() {
            let eval_count = stats.unwrap().frames[frame_i].eval_count.get();
            if eval_count == 0 {
                s.push_str(" (unused) \n");
                continue;
            }
        }

        s.push_str("\n");

        for channel in (0..4) {
            let skip_channel = stats.is_some()
                && stats.unwrap().frames[frame_i].channels[channel]
                    .eval_count
                    .get()
                    == 0;
            // println!(
            //     "skipping channel? ({}, {}), {}",
            //     frame_i, channel, skip_channel
            // );

            let gene_str = if skip_channel {
                "".to_owned()
            } else {
                render_genes(
                    &frame.channels[channel],
                    genetic_manifest,
                    stats.and_then(|stats| Some(&stats.frames[frame_i].channels[channel])),
                    2,
                )
            };

            let is_default = frame.default_channel as usize == channel;
            let default_str = if is_default {
                " (DEFAULT)".to_owned()
            } else {
                "".to_owned()
            };

            let pct_s = stats.map_or(String::new(), |stats| {
                display_pct(stats.frames[frame_i].channels[channel].pct_true())
            });

            let skip_str = if skip_channel {
                "(unused)".to_owned()
            } else {
                String::new()
            };

            s.push_str(&format!(
                "{}{}Channel #{} {}{}\n",
                indent_for_level(1),
                pct_s,
                channel,
                skip_str,
                &default_str
            ));

            if frame.channels[channel].len() > 0 {
                s.push_str(&format!("{}\n", &gene_str));
            }
        }
    }

    s
}

pub fn render_genes(
    genes: &Vec<Gene>,
    genetic_manifest: &GeneticManifest,
    stats: Option<&ChannelExecutionStats>,
    indent_level: usize,
) -> String {
    let mut s = String::new();

    for (gene_i, gene) in genes.iter().enumerate() {
        let gene_str = render_gene(
            gene,
            genetic_manifest,
            stats.and_then(|stats| Some(&stats.genes[gene_i])),
            indent_level,
        );
        s.push_str(&format!("{}", &gene_str));
    }

    s
}

pub fn render_gene(
    gene: &Gene,
    genetic_manifest: &GeneticManifest,
    stats: Option<&GeneExecutionStats>,
    indent_level: usize,
) -> String {
    let disjunctive_clause = &gene.conditional;
    let gene_op_call = &gene.operation;

    let pct_s = stats.map_or(String::new(), |stats| display_pct(stats.pct_true()));

    let conditional_str = if disjunctive_clause.conjunctive_clauses.len() == 0 {
        if disjunctive_clause.is_negated {
            " TRUE\n".to_owned()
        } else {
            " FALSE\n".to_owned()
        }
    } else {
        let clause_str = render_disjunction(
            disjunctive_clause,
            genetic_manifest,
            stats.map(|stats| &stats.disjunction_expression),
            indent_level + 1,
        );

        format!(":\n{}", clause_str)
    };

    let gene_op_str = render_gene_operation(&gene_op_call, genetic_manifest);
    format!(
        "{}{}CALL {} IF{}",
        indent_for_level(indent_level),
        pct_s,
        gene_op_str,
        conditional_str
    )
}

pub fn render_disjunction(
    dis: &Disjunction,
    genetic_manifest: &GeneticManifest,
    stats: Option<&DisjunctionExpressionStats>,
    indent_level: usize,
) -> String {
    let count = dis.conjunctive_clauses.len();
    let is_negated = dis.is_negated;
    let indent = indent_for_level(indent_level);

    let pct_s = stats.map_or(String::new(), |stats| display_pct(stats.pct_true()));

    let operation_line = format!(
        "{}{}{}:\n",
        indent,
        pct_s,
        if is_negated { "NOT OR" } else { "OR" }
    );

    let result = dis.conjunctive_clauses.iter().enumerate().fold(
        operation_line,
        |acc: String, pair: (usize, &Conjunction)| -> String {
            let (i, clause) = pair;

            format!(
                "{}{}",
                acc,
                render_conjunction(
                    clause,
                    genetic_manifest,
                    stats.map(|stats| &stats.conjunctive_expressions[i]),
                    indent_level + 1,
                    false,
                )
            )
        },
    );
    return result;
}

pub fn render_conjunction(
    clause: &Conjunction,
    genetic_manifest: &GeneticManifest,
    stats: Option<&ConjunctionExpressionStats>,
    indent_level: usize,
    flip_negation: bool,
) -> String {
    let is_negated = clause.is_negated;
    let indent = indent_for_level(indent_level);
    let pct_s = stats.map_or(String::new(), |stats| display_pct(stats.pct_true()));

    let count = clause.boolean_variables.len();

    let operation_line = format!(
        "{}{}{}:\n",
        indent,
        pct_s,
        if is_negated { "NOT AND" } else { "AND" }
    );

    // if count == 1 {
    //     return format!(
    //         "{}{}{}\n",
    //         indent,
    //         pct_s,
    //         clause.boolean_variables[0].render(genetic_manifest)
    //     );
    // }

    if count == 0 {
        let s = if is_negated {
            "FALSE".to_owned()
        } else {
            "TRUE".to_owned()
        };

        return format!("{}{}\n", indent_for_level(indent_level), s);
    }

    let result = clause.boolean_variables.iter().fold(
        operation_line,
        |acc: String, x: &BooleanVariable| -> String {
            let s = format!("{}", x.render(genetic_manifest));
            format!(
                "{}{}{}\n",
                acc,
                indent_for_level(indent_level + 1),
                &x.render(genetic_manifest)
            )
        },
    );

    // let is_negated_str = if is_negated { "NOT " } else { "" }.to_string();
    format!("{}", result)
}

pub fn render_gene_operation(
    call: &ParamedGeneOperationCall,
    genetic_manifest: &GeneticManifest,
) -> String {
    let sm = &genetic_manifest.sensor_manifest;
    match &call {
        ParamedGeneOperationCall::Reaction((reaction_id, p1, p2, p3)) => {
            let reaction = &genetic_manifest.chemistry_manifest.reactions[*reaction_id as usize];
            let required_count = genetic_manifest
                .chemistry_manifest
                .get_required_params_for_reaction(&reaction.key);

            if required_count == 0 {
                format!("{}()", reaction.key)
            } else if required_count == 1 {
                format!("{}({})", reaction.key, render_param(p1, sm))
            } else if required_count == 2 {
                format!(
                    "{}({}, {})",
                    reaction.key,
                    render_param(p1, sm),
                    render_param(p2, sm)
                )
            } else {
                format!(
                    "{}({}, {}, {})",
                    reaction.key,
                    render_param(p1, sm),
                    render_param(p2, sm),
                    render_param(p3, sm)
                )
            }
        }
        ParamedGeneOperationCall::MetaReaction(meta_reaction) => {
            format!("{}", meta_reaction.display(genetic_manifest))
        }
        ParamedGeneOperationCall::Nil => {
            format!("NilGeneOperationCall")
        }
    }
}
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

pub fn display_pct(pct: f32) -> String {
    if pct.is_nan() {
        return "[--%]  ".to_owned();
    }

    let s = if pct == 0.0 {
        "0".to_owned()
    } else if pct < 0.01 {
        format!("{:.2}", pct * 100.0)
    } else {
        let truncated = (pct * 100.0) as u8;
        truncated.to_string()
    };

    let len = s.len();

    if len == 2 {
        format!("[{}%]  ", s)
    } else {
        format!("[{}%] ", s)
    }
}

pub fn indent_for_level(indent_level: usize) -> String {
    " ".repeat(indent_level * INDENT_SIZE)
}

pub mod tests {
    use chemistry::variants::CheeseChemistry;
    // use crate::biology::framed::::{frame, framed_genome};

    use super::*;
    use crate::biology::genetic_manifest::predicates::default_operators;
    use crate::biology::genome::framed::common::*;
    // use crate::biology::genome::framed::*;
    use crate::biology::unit_behavior::framed::*;
    use crate::simulation::common::*;

    #[test]
    pub fn test_display_pct() {
        println!("{}", format!("{:.2}", 0.01));

        assert_eq!(display_pct(0.0010), "[0.10%] ");
    }

    #[test]
    pub fn test_render_genes() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let genes: Vec<Gene> = vec![
            Gene::new(
                Disjunction::new(
                    true,
                    vec![
                        Conjunction::new(
                            false,
                            vec![
                                BooleanVariable::Conditional(
                                    0,
                                    false,
                                    ParsedGenomeParam::Constant(0),
                                    ParsedGenomeParam::Constant(0),
                                    ParsedGenomeParam::Constant(0),
                                ),
                                BooleanVariable::Conditional(
                                    3,
                                    true,
                                    ParsedGenomeParam::Constant(0),
                                    ParsedGenomeParam::Constant(0),
                                    ParsedGenomeParam::Constant(0),
                                ),
                            ],
                        ),
                        Conjunction::new(true, vec![BooleanVariable::Literal(true)]),
                        Conjunction::new(false, vec![]),
                        Conjunction::new(true, vec![]),
                    ],
                ),
                ParamedGeneOperationCall::Reaction((
                    0,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            ),
            Gene::new(
                Disjunction::new(
                    false,
                    vec![
                        Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
                        Conjunction::new(false, vec![BooleanVariable::Literal(false)]),
                    ],
                ),
                ParamedGeneOperationCall::Reaction((
                    1,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            ),
            // with an empty disjunction (ie. automatically false)
            Gene::new(
                Disjunction::new(false, vec![]),
                ParamedGeneOperationCall::Reaction((
                    0,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            ),
            // with an empty disjunction (ie. automatically true because negated)
            Gene::new(
                Disjunction::new(true, vec![]),
                ParamedGeneOperationCall::Reaction((
                    0,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            ),
        ];

        println!("{}", render_genes(&genes, &gm, None, 0));

        let expected = "CALL gobble_cheese() IF:
    NOT OR:
        AND:
            Constant(0) == Constant(0)
            NOT Constant(0) > Constant(0)
        NOT AND:
            Value(true)
        TRUE
        FALSE
CALL move_unit(Constant(0)) IF:
    OR:
        AND:
            Value(true)
        AND:
            Value(false)
CALL gobble_cheese() IF FALSE
CALL gobble_cheese() IF TRUE\n";

        assert_eq!(expected, render_genes(&genes, &gm, None, 0));
    }

    #[test]
    pub fn test_render_genes_with_stats() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let frames = vec![Frame {
            channels: [
                vec![
                    Gene::new(
                        Disjunction::new(
                            false,
                            vec![
                                Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
                                Conjunction::new(false, vec![BooleanVariable::Literal(false)]),
                            ],
                        ),
                        ParamedGeneOperationCall::Reaction((
                            1,
                            ParsedGenomeParam::Constant(0),
                            ParsedGenomeParam::Constant(0),
                            ParsedGenomeParam::Constant(0),
                        )),
                    ),
                    // with an empty disjunction (ie. automatically false)
                    Gene::new(
                        Disjunction::new(false, vec![]),
                        ParamedGeneOperationCall::Reaction((
                            0,
                            ParsedGenomeParam::Constant(0),
                            ParsedGenomeParam::Constant(0),
                            ParsedGenomeParam::Constant(0),
                        )),
                    ),
                    // with an empty disjunction (ie. automatically true because negated)
                    Gene::new(
                        Disjunction::new(true, vec![]),
                        ParamedGeneOperationCall::Reaction((
                            0,
                            ParsedGenomeParam::Constant(0),
                            ParsedGenomeParam::Constant(0),
                            ParsedGenomeParam::Constant(0),
                        )),
                    ),
                ],
                vec![],
                vec![],
                vec![],
            ],
            default_channel: 0,
            address_range: (0, 0),
        }];

        let mut stats = FramedGenomeExecutionStats::empty();
        stats.initialize(&frames);
        stats.frames[0].eval_count.set(10);
        stats.frames[0].eval_true_count.set(5);

        stats.frames[0].channels[0].eval_count.set(10);
        stats.frames[0].channels[0].eval_true_count.set(5);

        stats.frames[0].channels[0].genes[0].eval_count.set(10);
        stats.frames[0].channels[0].genes[0].eval_true_count.set(5);

        stats.frames[0].channels[0].genes[1].eval_count.set(10);
        stats.frames[0].channels[0].genes[1].eval_true_count.set(5);

        stats.frames[0].channels[0].genes[2].eval_count.set(10);
        stats.frames[0].channels[0].genes[2].eval_true_count.set(5);

        let s = render_frames_with_stats(&frames, &gm, Some(&stats));
        println!("{}", &s);

        let expected = "[50%]   ***FRAME 0:***
    [50%]  Channel #0  (DEFAULT)
        [50%]  CALL move_unit(Constant(0)) IF:
            [--%]  OR:
                [--%]  AND:
                    Value(true)
                [--%]  AND:
                    Value(false)
        [50%]  CALL gobble_cheese() IF FALSE
        [50%]  CALL gobble_cheese() IF TRUE

    [--%]  Channel #1 (unused)
    [--%]  Channel #2 (unused)
    [--%]  Channel #3 (unused)\n";

        assert_eq!(expected, s);
    }
}
