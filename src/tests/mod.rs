use std::rc::Rc;

use crate::{
    biology::{
        genome::framed::{
            builders::{simple_convert_into_frames, FramedGenomeCompiler},
            common::CompiledFramedGenome,
        },
        unit_behavior::framed::FramedGenomeUnitBehavior,
    },
    simulation::common::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod,
        reactions::execute_reaction, ChemistryConfigBuilder, ChemistryConfiguration,
        ChemistryInstance, GeneticManifest, SimulationBuilder, UnitEntryBuilder, UnitManifest,
    },
};

pub fn make_sim(
    chemistry: ChemistryInstance,
    genome: Rc<CompiledFramedGenome>,
) -> SimulationBuilder {
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();
    SimulationBuilder::default()
        .chemistry(chemistry)
        .size((3, 3))
        .iterations(100)
        .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
            attributes: None,
            coords: vec![(0, 0)],
        })
        .unit_manifest(UnitManifest {
            units: vec![UnitEntryBuilder::default()
                .species_name("main".to_string())
                .behavior(FramedGenomeUnitBehavior::new(genome, gm.clone()).construct())
                .build(&gm.chemistry_manifest)],
        })
}

#[test]
pub fn default_chemistry_configuration() {
    let chemistry = ChemistryBuilder::with_key("foo").build();
    let val = chemistry
        .get_configuration()
        .get("magic_foo_unit_resource_amount")
        .unwrap()
        .unwrap_resource_amount();

    assert_eq!(val, 10);

    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();
    let cm = &gm.chemistry_manifest;

    let genome_values = genome!(gene(
        if_any(all((is_truthy, 1, 0, 0))),
        then_do(new_unit(0, 0, 0))
    ))
    .build(&gm);

    let framed_vals = simple_convert_into_frames(genome_values);
    let frames = FramedGenomeCompiler::compile(framed_vals, &gm);

    let mut sim = make_sim(chemistry, frames).to_simulation();

    // make sure the config values got populated all the way through
    assert_eq!(
        sim.chemistry
            .get_configuration()
            .get("magic_foo_unit_resource_amount")
            .unwrap()
            .unwrap_resource_amount(),
        10
    );
}

// test to make sure custom config values dont get overwritten
pub fn default_override_chemistry_configuration() {
    let target = 9999;

    let chemistry = ChemistryBuilder {
        chemistry_key: "foo".to_string(),
        chemistry_configuration: Some(
            ChemistryConfigBuilder::new()
                .set_integer("magic_foo_unit_resource_amount", target)
                .build(),
        ),
    }
    .build();

    let val = chemistry
        .get_configuration()
        .get("magic_foo_unit_resource_amount")
        .unwrap()
        .unwrap_integer();

    assert_eq!(val, target);

    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();
    let cm = &gm.chemistry_manifest;
    assert_eq!(cm.all_properties[1].id, 1); // make sure properties got created and normalized

    let genome_values = genome!(gene(
        if_any(all((is_truthy, 1, 0, 0))),
        then_do(set_foo_unit_resource_to_magic_amount(0, 0, 0))
    ))
    .build(&gm);

    let framed_vals = simple_convert_into_frames(genome_values);
    let frames = FramedGenomeCompiler::compile(framed_vals, &gm);

    let mut sim = make_sim(chemistry, frames).to_simulation();

    // make sure the config values got populated all the way through
    assert_eq!(
        sim.chemistry
            .get_configuration()
            .get("magic_foo_unit_resource_amount")
            .unwrap()
            .unwrap_integer(),
        target
    );

    sim.init();

    sim.tick();
    //
    use crate::chemistry::variants::foo::defs;
    let amount = sim.world.get_unit_resource_at(
        &(0, 0),
        defs::UnitResourcesLookup::new().foo_stored_resource,
    );

    assert_eq!(amount, target as i32);
}
