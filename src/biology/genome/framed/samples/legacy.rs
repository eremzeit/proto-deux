use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use crate::simulation::simulation_data::{
    new_threaded_simulation_reference, ThreadedSimulationReference,
};

use crate::biology::genome::framed::builders::legacy::util::GenomeBuilderLegacy;
use crate::biology::genome::framed::*;
use crate::biology::unit_behavior::framed::*;
use crate::simulation::common::UnitEntryBuilder;

pub fn get_genome1() -> GenomeBuilderLegacy {
    genome!(
        gene(
            if_any(all(
                (is_truthy, "pos_attr::is_cheese_dispenser(0, 0)", 0, 0),
                (gt, unit_res::cheese, 1000, 0)
            )),
            // move off the much needed spot
            then_do(move_unit(right))
        ),
        gene(
            if_any(all(
                (is_truthy, "pos_attr::is_cheese_dispenser(0, 0)", 0, 0),
                (gt, pos_res::milk, 300, 0)
            )),
            then_do(make_cheese(0))
        ),
        gene(
            if_any(all((gt, "pos_res::milk(0, 0)", 100, 0))),
            then_do(make_cheese(0))
        ),
        gene(
            if_any(all(
                (lt, unit_res::cheese, 60, 0),
                (gt, pos_res::milk, 20, 0)
            )),
            then_do(make_cheese(0))
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
            if_any(all((gt, unit_res::cheese, 600, 0))),
            then_do(new_unit(up))
        ),
        gene(
            if_any(all(
                (is_truthy, "pos_attr::is_cheese_dispenser(0, 0)", 0, 0),
                (lt, unit_res::cheese, 1000, 0)
            )),
            then_do(make_cheese(0))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(0, 1)", 0, 0)
            )),
            then_do(move_unit(up))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(1, 0)", 0, 0)
            )),
            then_do(move_unit(right))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(0, -1)", 0, 0)
            )),
            then_do(move_unit(down))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(-1, 0)", 0, 0)
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
        gene(
            if_any(all((true, random_hundred, 1, 0))),
            then_do(move_unit(up))
        )
    )
}
pub fn get_genome2() -> GenomeBuilderLegacy {
    genome!(
        gene(
            if_any(all(
                (is_truthy, "pos_attr::is_cheese_dispenser(0, 0)", 0, 0),
                (gt, unit_res::cheese, 1000, 0)
            )),
            // move off the much needed spot
            then_do(move_unit(up))
        ),
        gene(
            if_any(all(
                (is_truthy, "pos_attr::is_cheese_dispenser(0, 0)", 0, 0),
                (lt, pos_res::milk, 300, 0)
            )),
            then_do(make_cheese(0))
        ),
        gene(
            if_any(all(
                (lt, unit_res::cheese, 60, 0),
                (gt, pos_res::milk, 20, 0)
            )),
            then_do(make_cheese(0))
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
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(0, 1)", 0, 0)
            )),
            then_do(move_unit(up))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(1, 0)", 0, 0)
            )),
            then_do(move_unit(right))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(0, -1)", 0, 0)
            )),
            then_do(move_unit(down))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (is_truthy, "pos_attr::is_cheese_dispenser(-1, 0)", 0, 0)
            )),
            then_do(move_unit(left))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (gt, "pos_res::milk(0, 1)", 20, 0)
            )),
            then_do(move_unit(up))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (gt, "pos_res::milk(1, 0)", 20, 0)
            )),
            then_do(move_unit(right))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (gt, "pos_res::milk(0, -1)", 20, 0)
            )),
            then_do(move_unit(down))
        ),
        gene(
            if_any(all(
                (is_falsy, "pos_attr::is_cheese_dispenser", 0, 0),
                (gt, "pos_res::milk(-1, 0)", 20, 0)
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

pub fn get_genome3() -> GenomeBuilderLegacy {
    genome!(gene(
        if_any(all((true, 0, 0, 0))),
        then_do(move_unit(right))
    ))
}
