pub mod with_stats;

use super::types::*;
use super::types::{BooleanVariable, Conjunction};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::sensor_manifest::SensorManifest;
use crate::biology::unit_behavior::framed::{ParamedGeneOperationCall, ParsedGenomeParam};

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

pub fn render_genes(genes: &Vec<Gene>, genetic_manifest: &GeneticManifest) -> String {
    let mut s = String::new();

    for (gene_i, gene) in genes.iter().enumerate() {
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

pub fn render_conjunction(clause: &Conjunction, genetic_manifest: &GeneticManifest) -> String {
    let _items = clause.boolean_variables.iter();
    let is_negated = clause.is_negated;

    if _items.len() == 0 {
        return if is_negated {
            "FALSE".to_owned()
        } else {
            "TRUE".to_owned()
        };
    }

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

pub fn render_disjunction(dis: &Disjunction, genetic_manifest: &GeneticManifest) -> String {
    let _items = dis.conjunctive_clauses.iter();
    let is_negated = dis.is_negated;

    if _items.len() == 0 {
        return if is_negated {
            "TRUE".to_owned()
        } else {
            "FALSE".to_owned()
        };
    }

    let result = _items.fold("".to_string(), |acc: String, x: &Conjunction| -> String {
        if acc.len() == 0 {
            format!("{}", render_conjunction(x, genetic_manifest))
        } else {
            format!("( {} || {} )", acc, render_conjunction(x, genetic_manifest))
        }
    });

    let is_negated_str = if is_negated { "NOT " } else { "" }.to_string();
    format!("{}{}", is_negated_str, result)
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

#[cfg(test)]
pub mod tests {
    use crate::biology::genetic_manifest::GeneticManifest;
    use crate::biology::genome::framed::common::{BooleanVariable, Conjunction, Disjunction, Gene};
    use crate::biology::genome::framed::render::{
        render_conjunction, render_disjunction, render_gene, render_genes,
    };
    use crate::biology::unit_behavior::framed::ParamedGeneOperationCall;
    use crate::biology::unit_behavior::ParsedGenomeParam;
    use crate::chemistry::properties::CheeseChemistry;
    use crate::chemistry::{Chemistry, ChemistryConfiguration};

    #[test]
    pub fn conjunctive_to_str__simple() {
        let clause = Conjunction::new(
            false,
            vec![
                BooleanVariable::Literal(true),
                BooleanVariable::Literal(true),
                BooleanVariable::Literal(true),
            ],
        );

        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let result = render_conjunction(&clause, &gm);
        //println!("RESULT: {}", &result);
        assert_eq!(
            result,
            "((Value(true) && Value(true)) && Value(true))".to_string()
        );
    }

    #[test]
    pub fn conjunctive_to_str__conditional1() {
        let clause = Conjunction::new(
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

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let result = render_conjunction(&clause, &gm);

        assert_eq!(
            result,
            "(sim_attr::total_cheese_acquired(0, 0) == Constant(0) && Value(true))".to_string()
        );
    }

    #[test]
    pub fn conjunctive_to_str__conditional2() {
        let clause = Conjunction::new(
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

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let result = render_conjunction(&clause, &gm);

        assert_eq!(
            result,
            "(is_truthy(sim_attr::total_cheese_acquired(0, 0)) && Value(true))".to_string()
        );
    }

    #[test]
    pub fn disjunctive_to_str__simple() {
        let clause = Disjunction::new(
            false,
            vec![Conjunction::new(
                false,
                vec![BooleanVariable::Literal(true)],
            )],
        );

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let result = render_disjunction(&clause, &gm);

        assert_eq!(result, "Value(true)".to_string());
    }
    #[test]
    pub fn disjunctive_to_str__negated() {
        let clause = Disjunction::new(
            true,
            vec![Conjunction::new(
                false,
                vec![BooleanVariable::Literal(true)],
            )],
        );

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();
        let cm = CheeseChemistry::default_manifest();

        let result = render_disjunction(&clause, &gm);

        assert_eq!(result, "NOT Value(true)".to_string());
    }

    #[test]
    pub fn disjunctive_to_str__complex1() {
        let clause = Disjunction::new(
            true,
            vec![
                Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
                Conjunction::new(
                    true,
                    vec![
                        BooleanVariable::Literal(true),
                        BooleanVariable::Literal(false),
                    ],
                ),
            ],
        );

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();
        let result = render_disjunction(&clause, &gm);

        assert_eq!(
            result,
            "NOT ( Value(true) || NOT (Value(true) && Value(false)) )".to_string()
        );
    }

    #[test]
    pub fn render_gene__simple() {
        let disjunction = Disjunction {
            is_negated: false,
            conjunctive_clauses: vec![
                Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
                Conjunction::new(
                    false,
                    vec![
                        BooleanVariable::Literal(true),
                        BooleanVariable::Literal(false),
                    ],
                ),
            ],

            address_range: (0, 0),
        };
        let gene = Gene {
            conditional: disjunction,

            operation: ParamedGeneOperationCall::Nil,

            address_range: (0, 0),
        };

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

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
            Gene::new(
                Disjunction::new(
                    false,
                    vec![
                        Conjunction::new(false, vec![BooleanVariable::Literal(false)]),
                        Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
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
        ];

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let result = render_genes(&genes, &gm);
        let expected = "CALL make_cheese() IF ( Value(false) || Value(true) )
CALL move_unit(Constant(0)) IF ( Value(true) || Value(false) )\n";

        println!("RESULT: \n{}", &result);
        assert_eq!(result, expected);
    }

    #[test]
    pub fn render_genes__multi_channel() {
        let genes: Vec<Gene> = vec![
            Gene::new(
                Disjunction::new(
                    false,
                    vec![
                        Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
                        Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
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
                    true,
                    vec![
                        Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
                        Conjunction::new(false, vec![BooleanVariable::Literal(true)]),
                    ],
                ),
                ParamedGeneOperationCall::Reaction((
                    1,
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                    ParsedGenomeParam::Constant(0),
                )),
            ),
        ];

        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();

        let result = render_genes(&genes, &gm);
        let expected = "CALL make_cheese() IF ( Value(true) || Value(true) )
CALL move_unit(Constant(0)) IF NOT ( Value(true) || Value(true) )\n";

        assert_eq!(result, expected);
    }
}
