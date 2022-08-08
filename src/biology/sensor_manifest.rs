use simulation::common::{PropertyId, SimulationAttributes, World, Coord, ChemistryManifest, CoordOffset};
use util::{coord_by_coord_offset};
use std::rc::Rc;

pub type SensorValue = i32;

pub type SensorCoordOffset = CoordOffset;


pub struct SensorContext<'a> {
    pub world: &'a World,
    pub sim_attr: &'a SimulationAttributes,
    pub coord: &'a Coord,
}

impl<'a> SensorContext<'a> {
    pub fn from(world: &'a World, sim_attr: &'a SimulationAttributes, coord: &'a Coord) -> Self {
        SensorContext {
            world,
            sim_attr,
            coord,
        }
    }
}

pub type CustomSensorFunction = Rc<dyn Fn(&World, &SimulationAttributes, &SensorContext) -> SensorValue>;
use std;

#[derive(Clone)]
pub enum SensorType {
    // ie. UnitResource, UnitAttribute, PositionResource, PositionAttribute
    LocalChemicalProperty(PropertyId, SensorCoordOffset),

    SimulationProperty(PropertyId),
    Constant(SensorValue),
    Random(std::ops::Range<usize>),
    Custom(CustomSensorFunction),
}

pub type SensorId = usize;

#[derive(Clone)]
pub struct SensorDefinition {
    pub id: SensorId,
    pub key: String,
    pub prop_key: String,
    pub sensor_type: SensorType,
}

impl SensorDefinition {
    pub fn calculate(&self, context: &SensorContext) -> SensorValue {
        return match &self.sensor_type {
            SensorType::LocalChemicalProperty(prop_id, coord_offset) => {
                if let Some(prop_val) = calc_local_chemical_property(prop_id, coord_offset, context) {
                    prop_val
                } else {
                    0
                }
            },
            SensorType::Constant(val) => { *val as i32 },
            
            SensorType::SimulationProperty(prop_id) => {
                context.sim_attr[prop_id.coerce_to_sim_attribute_id()].coerce_unwrap_to_integer()
            },
            SensorType::Random(range) => {
                use rand::Rng;

                // HTNS: this is probably slow
                let mut rng = rand::thread_rng();
                use std::convert::TryInto; 
                rng.gen_range(range.clone()).try_into().unwrap()
              },
            SensorType::Custom(func) => { 0 }
        };

        0
    }


    pub fn can_pre_calculate(&self) -> bool {
        match &self.sensor_type {
            SensorType::Constant(_) => true,
            _ => false,
        }
    }
}

pub fn calc_local_chemical_property(prop_id: &PropertyId, coord_offset: &CoordOffset, context: &SensorContext) -> Option<SensorValue> {
    let coord = match coord_by_coord_offset(context.coord, coord_offset.clone(), context.world.size.clone()) {
        Some(coord) => { coord },
        None => { return None; }
    };

    match prop_id {
        PropertyId::PositionAttributeId(idx) => return Some(context.world.get_pos_attribute_at(&coord, *idx).coerce_unwrap_to_integer()),
        PropertyId::PositionResourceId(idx) => return Some(context.world.get_pos_resource_at(&coord, *idx)),
        PropertyId::SimulationAttributeId(idx) => return Some(context.sim_attr[*idx as usize].coerce_unwrap_to_integer()),
        _ => {}
    };
    
    match prop_id {
        PropertyId::UnitAttributeId(idx) => return Some(context.world.get_unit_attribute_at(&coord, *idx).coerce_unwrap_to_integer()),
        PropertyId::UnitResourceId(idx) => return Some(context.world.get_unit_resource_at(&coord, *idx)),
        _ => {},
    };

    None
    
    // NOT: Not every local property will be supported.  ie. the concept of limited vision
    // UnitAttributeId(UnitAttributeIndex),
    // PositionAttributeId(PositionAttributeIndex),
    // UnitResourceId(UnitResourceIndex),
    // PositionResourceId(PositionResourceIndex),
    // SimulationAttributeId(SimulationAttributeIndex)
}

#[derive(Clone)]
// is this on a per-species basis? or does it describe the entire simulation?
pub struct SensorManifest {
    pub sensors: Vec<SensorDefinition>,
}

impl SensorManifest {
    pub fn with_default_sensors(chemistry_manifest: &ChemistryManifest) -> Self {
        Self::new(Self::default_sensors(chemistry_manifest))
    }

    pub fn new(sensors: Vec<SensorDefinition>) -> Self {
        SensorManifest {
            sensors,
        }
    }

    pub fn normalize_sensors(_sensors: Vec<SensorDefinition>) -> Vec<SensorDefinition> {
        _sensors.iter().enumerate().map(|(i, s)| { 
            let mut sensor = s.clone();
            sensor.id = i; 
            return sensor;
        }).collect::<Vec<_>>()
    }

    pub fn default_sensors(chemistry_manifest: &ChemistryManifest) -> Vec<SensorDefinition> {
        let offsets = sensor_local_offsets(1);        
        let mut sensors = vec![];

        for (i, offset) in offsets.iter().enumerate() {
            let mut _sensors: Vec<SensorDefinition> = chemistry_manifest.all_properties.iter().map(|prop| {
                SensorDefinition {
                    key: format!("{}{:?}", &prop.long_key, &offset),
                    prop_key: prop.long_key.clone(),
                    id: 0,
                    sensor_type: SensorType::LocalChemicalProperty(prop.property_id.clone(), offset.clone()),
                }
            }).collect::<Vec<_>>();

            sensors.append(&mut _sensors);
        }

        sensors.push(SensorDefinition {
            id: 0,
            key: "random_val".to_string(),
            sensor_type: SensorType::Random(0..SensorValue::MAX as usize),
            prop_key: "random_val".to_string(),
        });
        
        sensors.push(SensorDefinition {
            id: 0,
            key: "random_hundred".to_string(),
            sensor_type: SensorType::Random(0..100 as usize),
            prop_key: "random_hundred".to_string(),
        });
        
        sensors.push(SensorDefinition {
            id: 0,
            key: "random_bool".to_string(),
            sensor_type: SensorType::Random(0..2),
            prop_key: "random_bool".to_string(),
        });

        Self::normalize_sensors(sensors)
    }
    
    pub fn sensor_id_from_key<T: AsRef<str>>(&self, _key: T) -> SensorId {
        self.identify_sensor_from_key(_key).unwrap().id
    }
    
    pub fn identify_sensor_from_key<T: AsRef<str>>(&self, _key: T) -> Option<SensorDefinition> {
        let mut key = _key.as_ref().clone().trim_end_matches('"').trim_start_matches('"').to_string();
        //println!("identify_sensor_from_key: {}", &key);

        let original_key = key.clone();
        if !key.ends_with(")") {
            key = format!("{}{}", &key, "(0, 0)")
        }

        for (i, sensor) in self.sensors.iter().enumerate() {
            //println!("{}, {}", &sensor.key, &sensor.prop_key);

            //if key == &sensor.key || key == &sensor.prop_key { return Some(sensor.clone()) }
            if &key == &sensor.key || &original_key == &sensor.key { 
                return Some(sensor.clone()) 
            }
        }

        return None;
    }
}

pub fn sensor_local_offsets(distance: i32) -> Vec<SensorCoordOffset> {
    let mut coords: Vec<SensorCoordOffset> = vec![];

    if distance == 0 {
        return vec![(0,0)];
    } else {
        let mut coords: Vec<(i32, i32)> = vec![];

        //xxx
        //xxx
        //xxx

        //top (LTR)
        for i in (-distance + 1)..=distance {
            coords.push((i, distance));
        }

        //right (TTB)
        for i in (-distance + 1)..=distance {
            coords.push((distance, -i));
        }

        //bottom (RTL)
        for i in (-distance + 1)..=distance {
            coords.push((-i, -distance));
        }
        //left (BTT)
        for i in (-distance + 1)..=distance {
            coords.push((-distance, i));
        }

        let mut new_coords = sensor_local_offsets(distance - 1);
        new_coords.append(&mut coords);
        return new_coords;
    }
}

pub mod tests {
    use super::{*};

    #[test]
    fn test_sensor_local_coords() {
        assert_eq!(sensor_local_offsets(0), vec![(0,0)]);
        assert_eq!(sensor_local_offsets(1), vec![(0, 0), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)]);
        assert_eq!(sensor_local_offsets(2), vec![(0, 0), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1), (-1, 2), (0, 2), (1, 2), (2, 2), (2, 1), (2, 0), (2, -1), (2, -2), (1, -2), (0, -2), (-1, -2), (-2, -2), (-2, -1), (-2, 0), (-2, 1), (-2, 2)]);
    }
}