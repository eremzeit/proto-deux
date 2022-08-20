use crate::chemistry::variants::cheese::*;
use crate::chemistry::*;
use crate::simulation::common::*;
use crate::simulation::*;
use crate::util::text_grid::{render_into_grid, GridCellRenderer};

pub struct ResourceGridCellRenderer {
    pub resource_idx: UnitResourceIndex,
}

impl<'a> ResourceGridCellRenderer {
    pub fn new(resource_idx: UnitResourceIndex) -> Self {
        ResourceGridCellRenderer { resource_idx }
    }
}

impl GridCellRenderer for ResourceGridCellRenderer {
    fn render(&self, coord: &Coord, world: &World) -> String {
        let pos = world.get_position_at(coord).unwrap();

        if pos.has_unit() {
            let amount = world.get_unit_resource_at(coord, self.resource_idx);
            format!("{}", amount)
        } else {
            format!("~")
        }
    }
}

mod tests {
    use crate::simulation::common::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod,
    };

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn render_amounts() {
        let chemistry = ChemistryBuilder::with_key("cheese").build();

        let mut sim = SimulationBuilder::default()
            .place_units_method(PlaceUnitsMethod::LinearBottomMiddle { attributes: None })
            .chemistry(chemistry)
            .size((2, 2))
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
            .to_simulation();

        let renderer = ResourceGridCellRenderer {
            resource_idx: sim
                .chemistry
                .get_manifest()
                .unit_resource_by_key("cheese")
                .id as UnitResourceIndex,
        };

        let options = TextGridOptions {
            cell_width: 5,
            cell_height: 5,
            has_border: true,
            alignment: CellTextAlignment::Center,
        };

        let r: Box<dyn GridCellRenderer> = Box::new(renderer);
        let s = render_into_grid(options, &r, &sim.world);
        let e = "
-------------
|  ~  |  ~  |
|     |     |
|     |     |
|     |     |
|     |     |
-------------
|  0  |  ~  |
|     |     |
|     |     |
|     |     |
|     |     |
-------------"
            .trim()
            .to_string();
        assert_eq!(s.trim().to_string(), e.to_string());
    }
}
