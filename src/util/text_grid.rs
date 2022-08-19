use crate::simulation::common::*;
use crate::simulation::iterators::CoordIterator;
use crate::util::GridSize2D;

use pad::PadStr;

pub struct TextGridOptions {
    pub cell_width: usize,
    pub cell_height: usize,
    pub has_border: bool,
    pub alignment: CellTextAlignment,
}

pub fn render_into_grid(
    options: TextGridOptions,
    renderer: &Box<dyn GridCellRenderer>,
    world: &World,
) -> String {
    let v_border_char = '|';
    let h_border_char = '-';

    let size_x = world.size.0;
    let size_y = world.size.1;

    let total_width: usize = if options.has_border {
        (options.cell_width + 1) * size_x + 1
    } else {
        options.cell_width * size_y
    };

    let mut buffer = String::new();

    if (options.has_border) {
        buffer += &("".pad(total_width, '-', pad::Alignment::Left, true) + "\n");
    }

    for y_off in 0..size_y {
        let mut lines: Vec<Vec<String>> = vec![];
        for x in 0..size_x {
            let y = size_y - 1 - y_off;
            println!("{}, {}, {}", x, y, y_off);
            let cell_str = renderer.render(&(x, y), world);
            let a = cell_str
                .split("\n")
                .map(|x| -> String { x.to_string() })
                .collect::<Vec<_>>();
            lines.push(a);
        }

        for cell_line in 0..options.cell_height {
            if (options.has_border) {
                buffer += "|";
            }

            for x in 0..size_x {
                let s = if cell_line < lines[x].len() {
                    lines[x][cell_line].to_string()
                } else {
                    "".to_string()
                };
                //println!("what: {}", options.cell_width);
                //println!("here: {}", s);

                let alignment = match options.alignment {
                    CellTextAlignment::Center => pad::Alignment::Middle,
                    CellTextAlignment::Left => pad::Alignment::Left,
                    CellTextAlignment::Right => pad::Alignment::Right,
                };

                buffer += &s.pad(options.cell_width, ' ', alignment, true);

                if (options.has_border) {
                    buffer += "|";
                }
            }

            buffer += "\n";
        }

        if (options.has_border) {
            buffer += &("".pad(total_width, h_border_char, pad::Alignment::Left, true) + "\n");
        }
    }

    buffer
}

pub trait GridCellRenderer {
    fn render(&self, coord: &Coord, world: &World) -> String;
}

pub fn multiline_trim(string: String) -> String {
    let lines = string
        .split("\n")
        .map(|x| -> String { x.trim().to_string() })
        .collect::<Vec<_>>();

    let mut buffer = String::new();

    for i in 0..lines.len() {
        buffer += &lines[i].to_string();
        if i < lines.len() - 1 {
            buffer += &"\n";
        }
    }

    buffer
}

pub enum CellTextAlignment {
    Center,
    Left,
    Right,
}

mod tests {
    struct TestGridCellRenderer {}

    impl GridCellRenderer for TestGridCellRenderer {
        fn render(&self, coord: &Coord, world: &World) -> String {
            format!("{},{}", coord.0, coord.1)
        }
    }

    #[allow(unused_imports)]
    use super::*;
    use crate::{
        chemistry::variants::cheese::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
    };

    #[test]
    fn render_coords_with_border() {
        let renderer: Box<dyn GridCellRenderer> = Box::new(TestGridCellRenderer {});
        let options = TextGridOptions {
            cell_width: 5,
            cell_height: 5,
            has_border: true,
            alignment: CellTextAlignment::Center,
        };

        let specs = SimulationSpecs {
            chemistry_key: "cheese".to_string(),
            place_units_method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
            ..Default::default()
        };

        let mut sim = SimulationBuilder::default()
            .size((2, 2))
            .specs(specs)
            .headless(true)
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
            .to_simulation();

        let s = render_into_grid(options, &renderer, &sim.world);
        let e = "
-------------
| 0,1 | 1,1 |
|     |     |
|     |     |
|     |     |
|     |     |
-------------
| 0,0 | 1,0 |
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
