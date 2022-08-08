pub mod legacy;

use crate::biology::genome::framed::common::*;
use crate::biology::phenotype::framed::common::*;
use crate::simulation::common::*;

use crate::biology::genome::framed::builders::*;

pub fn genome1(cm: &ChemistryManifest, sm: &SensorManifest, gm: &GeneticManifest) -> FramedGenome {
    let framed_vals = frame(
        0,
        vec![
            gene(
                if_any(vec![if_all(vec![
                    conditional!(is_truthy, pos_attr::is_cheese_source(0, 0)),
                    conditional!(gt, unit_res::cheese, 100),
                ])]),
                then_do!(move_unit, up),
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

    FramedGenomeParser::parse(framed_vals, cm.clone(), sm.clone(), gm.clone())
}

#[macro_export]
macro_rules! if_any {
    ($($x:expr),+) => ({
        if_any(vec![
            $($x),+
        ])
    })
}

#[macro_export]
macro_rules! if_all {
    ($($x:expr),+) => ({
        {
            let mut v = vec![
                $($x),+
            ];

            if_all(v)
        }
    })
}

pub fn get_genome1(
    cm: &ChemistryManifest,
    sm: &SensorManifest,
    gm: &GeneticManifest,
) -> FramedGenome {
    let framed_vals = frame(
        0,
        vec![
            gene(
                if_any!(if_all!(
                    conditional!(is_truthy, pos_attr::is_cheese_source(0, 0)),
                    conditional!(gt, unit_res::cheese, 1000)
                )),
                then_do!(move_unit, right),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_truthy, pos_attr::is_cheese_source(0, 0)),
                    conditional!(gt, unit_res::cheese, 300)
                )),
                then_do!(gobble_cheese),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(lt, unit_res::cheese, 60),
                    conditional!(gt, pos_res::cheese, 20)
                )),
                then_do!(gobble_cheese),
            ),
            gene(
                if_any!(if_all!(conditional!(lt, random_hundred, 20))),
                then_do!(move_unit, random(4)),
            ),
            gene(
                if_any!(if_all!(conditional!(gt, unit_res::cheese, 600))),
                then_do!(new_unit, up),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_truthy, pos_attr::is_cheese_source),
                    conditional!(lt, unit_res::cheese, 1000)
                )),
                then_do!(gobble_cheese),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_falsy, pos_attr::is_cheese_source),
                    conditional!(is_truthy, pos_attr::is_cheese_source(0, 1))
                )),
                then_do!(move_unit, up),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_falsy, pos_attr::is_cheese_source),
                    conditional!(is_truthy, pos_attr::is_cheese_source(0, 1))
                )),
                then_do!(move_unit, up),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_falsy, pos_attr::is_cheese_source),
                    conditional!(is_truthy, pos_attr::is_cheese_source(0, 1))
                )),
                then_do!(move_unit, up),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_falsy, pos_attr::is_cheese_source),
                    conditional!(is_truthy, pos_attr::is_cheese_source(1, 0))
                )),
                then_do!(move_unit, right),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_falsy, pos_attr::is_cheese_source),
                    conditional!(is_truthy, pos_attr::is_cheese_source(-1, 0))
                )),
                then_do!(move_unit, left),
            ),
            gene(
                if_any!(if_all!(
                    conditional!(is_falsy, pos_attr::is_cheese_source),
                    conditional!(is_truthy, pos_attr::is_cheese_source(0, -1))
                )),
                then_do!(move_unit, down),
            ),
        ],
    )
    .build(&sm, &cm, &gm);

    FramedGenomeParser::parse(framed_vals, cm.clone(), sm.clone(), gm.clone())
}
pub fn get_genome2() -> GenomeBuilder {
    genome!(
        gene(
            if_any(all(
                (is_truthy, "pos_attr::is_cheese_source(0, 0)", 0, 0),
                (gt, unit_res::cheese, 1000, 0)
            )),
            // move off the much needed spot
            then_do(move_unit(up))
        ),
        gene(
            if_any(all(
                (is_truthy, "pos_attr::is_cheese_source(0, 0)", 0, 0),
                (lt, pos_res::cheese, 300, 0)
            )),
            then_do(gobble_cheese(0))
        ),
        gene(
            if_any(all(
                (lt, unit_res::cheese, 60, 0),
                (gt, pos_res::cheese, 20, 0)
            )),
            then_do(gobble_cheese(0))
        ),
        gene(
            if_any(all((lt, random_hundred, 10, 0))),
            then_do(move_unit(up))
        ),
        gene(
            if_any(all((lt, random_hundred, 10, 0))),
            then_do(move_unit(right))
        ),
        gene(
            if_any(all((lt, random_hundred, 10, 0))),
            then_do(move_unit(down))
        ),
        gene(
            if_any(all((lt, random_hundred, 20, 0))),
            then_do(move_unit(left))
        ),
        gene(
            if_any(all((gt, unit_res::cheese, 700, 0))),
            then_do(new_unit(up))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (is_truthy, "pos_attr::is_cheese_source(0, 1)", 0, 0)
            )),
            then_do(move_unit(up))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (is_truthy, "pos_attr::is_cheese_source(1, 0)", 0, 0)
            )),
            then_do(move_unit(right))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (is_truthy, "pos_attr::is_cheese_source(0, -1)", 0, 0)
            )),
            then_do(move_unit(down))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (is_truthy, "pos_attr::is_cheese_source(-1, 0)", 0, 0)
            )),
            then_do(move_unit(left))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (gt, "pos_res::cheese(0, 1)", 20, 0)
            )),
            then_do(move_unit(up))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (gt, "pos_res::cheese(1, 0)", 20, 0)
            )),
            then_do(move_unit(right))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (gt, "pos_res::cheese(0, -1)", 20, 0)
            )),
            then_do(move_unit(down))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_source", 0, 0),
                (gt, "pos_res::cheese(-1, 0)", 20, 0)
            )),
            then_do(move_unit(left))
        ),
        gene(
            if_any(all((lt, random_hundred, 20, 0))),
            then_do(move_unit(up))
        ),
        gene(
            if_any(all((lt, random_hundred, 20, 0))),
            then_do(move_unit(right))
        ),
        gene(
            if_any(all((lt, random_hundred, 20, 0))),
            then_do(move_unit(down))
        ),
        gene(
            if_any(all((lt, random_hundred, 20, 0))),
            then_do(move_unit(left))
        ),
        gene(
            if_any(all((lt, random_hundred, 50, 0))),
            then_do(move_unit(right))
        ),
        gene(if_any(all((true, 0, 0, 0))), then_do(move_unit(up)))
    )
}

use crate::biology::genome::framed::builders::legacy::util::*;

pub fn get_genome3() -> GenomeBuilder {
    genome!(gene(
        if_any(all((true, 0, 0, 0))),
        then_do(move_unit(right))
    ))
}
