use super::ActionParam;
use std::collections::HashMap;

use std::rc::Rc;

use crate::chemistry::variants::nanobots::NanobotsChemistry;
use crate::simulation::common::*;
use crate::simulation::position::{
    PositionAttributeIndex, PositionAttributeValue, PositionResourceAmount, PositionResourceIndex,
};
use crate::simulation::unit::{
    UnitAttributeIndex, UnitAttributeValue, UnitResourceAmount, UnitResourceIndex,
};

use crate::util::*;

use crate::util::{coord_by_direction_offset, Coord};

use crate::chemistry::actions::default_actions;
use crate::simulation::common::{GridDirection, UnitEntry};

pub mod set_unit_resource {
    use crate::simulation::common::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod,
    };

    use super::*;

    #[test]
    fn test__evaluate() {
        let chemistry_builder = ChemistryBuilder::with_key("nanobots");

        let mut sim = SimulationBuilder::default()
            .chemistry(chemistry_builder.build())
            .size((5, 5))
            .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(1, 1)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
            .to_simulation();

        let params = vec![
            ActionParam::UnitResourceIndex(0),
            ActionParam::UnitResourceAmount(10),
        ];

        let actions = default_actions();
        let action = actions
            .iter()
            .find(|a| a.key == "set_unit_resource")
            .unwrap();

        assert!(execute_action(
            &action,
            &(1, 1),
            &mut sim,
            params.as_slice()
        ));
    }
}

pub mod offset_unit_resource {
    use super::*;
    use crate::{
        chemistry::variants::cheese,
        simulation::common::{builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod},
    };

    #[test]
    fn test_evaluate_strict() {
        let chemistry_builder = ChemistryBuilder::with_key("nanobots");

        let mut sim = SimulationBuilder::default()
            .chemistry(chemistry_builder.build())
            .size((5, 5))
            .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(2, 2)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
            .to_simulation();

        let unit_resources = cheese::defs::UnitResourcesLookup::new();
        sim.world
            .set_unit_resource_at(&(2, 2), unit_resources.cheese, 0);
        let params = vec![
            constant_arg!(UnitResourceIndex, unit_resources.cheese),
            constant_arg!(UnitResourceAmount, -10), // <-- expresses how much resources it will attempt to offset
            constant_arg!(Boolean, false),
        ];

        let actions = default_actions();
        let action = actions
            .iter()
            .find(|a| a.key == "offset_unit_resource")
            .unwrap();

        // shouldn't execute because not enough resources
        assert!(!execute_action(&action, &(2, 2), &mut sim, &params));

        sim.world
            .set_unit_resource_at(&(2, 2), unit_resources.cheese, 10);

        assert!(execute_action(&action, &(2, 2), &mut sim, &params));
    }
}

pub mod grow_unit {
    use super::*;
    use crate::fixtures;
    use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
    use crate::util::*;
    fn test_new_unit(src_coord: Coord, dir: GridDirection) {
        let actions = default_actions();
        let action = actions.iter().find(|a| a.key == "new_unit").unwrap();

        let mut sim =
            fixtures::default_base_with_unit_placement(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![src_coord],
            });

        assert_eq!(
            sim.world.get_position_at(&src_coord).unwrap().has_unit(),
            true
        );
        let direction = dir;
        let params = vec![ActionParam::Direction(direction.clone())];
        let result = execute_action(&action, &src_coord, &mut sim, &params);
        assert_eq!(result, true, "The action failed");
        let dest_coord = coord_by_direction_offset(&src_coord, &direction, sim.world.size).unwrap();

        assert_unit_at!(sim, &dest_coord);
        assert_unit_at!(sim, &src_coord);
    }

    #[test]
    fn grow_unit_left() {
        // TODO
        test_new_unit((2, 0), GridDirection::Up);
        // test_new_unit((0,0), GridDirection::Right);
        // test_new_unit((4,1), GridDirection::Down);
        // test_new_unit((4,4), GridDirection::Left);
    }
}

// pub fn can_execute(action: &ActionDefinition, coord: &Coord, simulation: &Simulation, params: &[ActionParam]) -> bool {
//
//     //unit_manifest: &UnitManifest, chemistry: &ChemistryInstance
//     let context = ActionExecutionContext {
//         coord,
//         params,
//         unit_manifest: &simulation.unit_manifest,
//         chemistry: &simulation.chemistry,
//     };
//
//     (action.can_execute)(
//         &simulation.world,
//         &simulation.attributes,
//         &context,
//     )
// }

pub fn execute_action(
    action: &ActionDefinition,
    coord: &Coord,
    simulation: &mut Simulation,
    params: &[ActionParam],
) -> bool {
    let context = ActionExecutionContext {
        coord,
        params,
        // unit_manifest: &simulation.unit_manifest,
        // chemistry: &simulation.chemistry,
    };

    (action.execute)(&mut simulation.editable(), &context)
}
