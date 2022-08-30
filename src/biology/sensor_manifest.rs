use serde::{Deserialize, Serialize};

use crate::simulation::common::{
    Chemistry, ChemistryManifest, Coord, CoordOffset, Property, PropertyId, SimulationAttributes,
    World,
};
use crate::util::coord_by_coord_offset;
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

pub type CustomSensorLibrary = Vec<CustomSensorImplementation>;

pub struct CustomSensorImplementation {
    pub sensor_fn: CustomSensorFunction,
    pub key: String,
}

pub type CustomSensorFunction =
    Rc<dyn Fn(&World, &SimulationAttributes, &SensorContext) -> SensorValue>;
use std;

pub type CustomSensorId = u16;
pub type CustomSensorKey = u16;

#[derive(Clone, Serialize, Deserialize)]
pub enum SensorType {
    // ie. UnitResource, UnitAttribute, PositionResource, PositionAttribute
    LocalChemicalProperty(PropertyId, SensorCoordOffset),
    SimulationProperty(PropertyId),
    Constant(SensorValue),
    Random(std::ops::Range<usize>),
    CustomSensor(CustomSensorKey),

    #[serde(skip_serializing, skip_deserializing)]
    CustomSensorFn(CustomSensorFunction, CustomSensorKey), // TODO: this should hold an ID to a preset list of custom sensor functions
}

pub type SensorId = usize;

/**
 * A sensor definition is a mapping between a genome sensor_id and how that sensor
 * gets executed within the context of that specific unit_entry and chemistry.  This means that
 * definitions need to be generated dynamically at the time that a unit_entry is initialized.
 *
 */
#[derive(Clone, Serialize, Deserialize)]
pub struct SensorDefinition {
    pub id: SensorId,
    pub key: String,
    pub prop_key: String, //for display purposes.  is used to store the original property key that corresponds to this sensor
    pub sensor_type: SensorType,
}

impl SensorDefinition {
    pub fn calculate(&self, context: &SensorContext) -> SensorValue {
        return match &self.sensor_type {
            SensorType::LocalChemicalProperty(prop_id, coord_offset) => {
                if let Some(prop_val) = calc_local_chemical_property(prop_id, coord_offset, context)
                {
                    prop_val
                } else {
                    0
                }
            }
            SensorType::Constant(val) => *val as i32,

            SensorType::SimulationProperty(prop_id) => {
                context.sim_attr[prop_id.coerce_to_sim_attribute_id()].coerce_unwrap_to_integer()
            }
            SensorType::Random(range) => {
                use rand::Rng;

                // AOEU: this is probably slow
                let mut rng = rand::thread_rng();
                use std::convert::TryInto;
                rng.gen_range(range.clone()).try_into().unwrap()
            }
            SensorType::CustomSensorFn(func, custom_sensor_key) => {
                panic!("not implemented")
            }
            SensorType::CustomSensor(custom_sensor_key) => {
                panic!("not implemented")
            }
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

pub fn calc_local_chemical_property(
    prop_id: &PropertyId,
    coord_offset: &CoordOffset,
    context: &SensorContext,
) -> Option<SensorValue> {
    let coord = match coord_by_coord_offset(
        context.coord,
        coord_offset.clone(),
        context.world.size.clone(),
    ) {
        Some(coord) => coord,
        None => {
            return None;
        }
    };

    match prop_id {
        PropertyId::PositionAttributeId(idx) => {
            return Some(
                context
                    .world
                    .get_pos_attribute_at(&coord, *idx)
                    .coerce_unwrap_to_integer(),
            )
        }
        PropertyId::PositionResourceId(idx) => {
            return Some(context.world.get_pos_resource_at(&coord, *idx))
        }
        PropertyId::SimulationAttributeId(idx) => {
            return Some(context.sim_attr[*idx as usize].coerce_unwrap_to_integer())
        }
        _ => {}
    };

    match prop_id {
        PropertyId::UnitAttributeId(idx) => {
            if !context.world.has_unit_at(&coord) {
                return None;
            } else {
                return Some(
                    context
                        .world
                        .get_unit_attribute_at(&coord, *idx)
                        .coerce_unwrap_to_integer(),
                );
            }
        }
        PropertyId::UnitResourceId(idx) => {
            if context.world.has_unit_at(&coord) {
                return Some(context.world.get_unit_resource_at(&coord, *idx));
            } else {
                return None;
            }
        }
        _ => {}
    };

    None

    // NOT: Not every local property will be supported.  ie. the concept of limited vision
    // UnitAttributeId(UnitAttributeIndex),
    // PositionAttributeId(PositionAttributeIndex),
    // UnitResourceId(UnitResourceIndex),
    // PositionResourceId(PositionResourceIndex),
    // SimulationAttributeId(SimulationAttributeIndex)
}

pub type SensorManifestData = SensorManifest;

/**
 * A sensor manifest is a list of sensors that are available to a specific unit_entry.  Over the course of the lifetime of
 * a genome, the sensor manifest cannot change.  This would mean that if we added/removed/changed any sensors to the manifest
 * then the genome would become invalid.  This means that tweaking the available sensors would invalidate any genomes that are currently stored.
 */
#[derive(Clone, Serialize, Deserialize)]
pub struct SensorManifest {
    pub sensors: Vec<SensorDefinition>,
}

impl SensorManifest {
    // pub fn to_compiled_sensor_manifest(
    //     &self,
    //     custom_sensor_library: Vec<CustomSensorImplementation>,
    // ) -> CompiledSensorManifest {
    //     // replace each custom sensor defininition with the compiled implementation
    //     todo!();
    //     self.clone()
    // }

    // pub fn with_default_sensors(chemistry_manifest: &ChemistryManifest) -> Self {
    //     SensorManifest {
    //         sensors: Self::default_sensors(chemistry_manifest),
    //     }
    // }

    pub fn new(
        chemistry_manifest: &ChemistryManifest,
        local_properties: &LocalPropertySensorManifest,
    ) -> Self {
        let mut sensors =
            Self::construct_local_property_sensors(local_properties, chemistry_manifest);

        sensors.append(&mut Self::standard_sensors(chemistry_manifest));

        SensorManifest {
            sensors: Self::normalize_sensors(sensors),
        }
    }

    pub fn normalize_sensors(_sensors: Vec<SensorDefinition>) -> Vec<SensorDefinition> {
        _sensors
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut sensor = s.clone();
                sensor.id = i;
                return sensor;
            })
            .collect::<Vec<_>>()
    }

    // pub construct_sensors() -> Vec<SensorDefinition> {

    // }

    pub fn construct_local_property_sensors(
        local_prop_manifest: &LocalPropertySensorManifest,
        chemistry_manifest: &ChemistryManifest,
    ) -> Vec<SensorDefinition> {
        let mut defs = vec![];

        for local_prop_entry in &local_prop_manifest.entries {
            let property = &chemistry_manifest.all_properties[local_prop_entry.property_offset_idx];

            let offsets = sensor_local_offsets(local_prop_entry.distance.into());

            let mut _defs = offsets
                .iter()
                .map(|offset| SensorDefinition {
                    key: format!("{}{:?}", &property.long_key, &offset),
                    prop_key: property.long_key.clone(),
                    id: 0,
                    sensor_type: SensorType::LocalChemicalProperty(
                        property.property_id.clone(),
                        offset.clone(),
                    ),
                })
                .collect::<Vec<_>>();

            defs.append(&mut _defs);
        }

        defs
        // chemistry_manifest
        //     .all_properties
        //     .iter()
        //     .map(|prop| SensorDefinition {
        //         key: format!("{}{:?}", &prop.long_key, &offset),
        //         prop_key: prop.long_key.clone(),
        //         id: 0,
        //         sensor_type: SensorType::LocalChemicalProperty(
        //             prop.property_id.clone(),
        //             offset.clone(),
        //         ),
        //     })
        //     .collect::<Vec<_>>();
    }

    /**
     *  Construct a list of sensors available from the chemistry manifest.
     *  Note, this assumes that *all* chemical properties are visible to the genome.
     *
     * We might eventually want to construct a sensor manifest that has only a subset of
     * chemical properties.  This would require passing a list of property keys that are to be
     * included/excluded.  Each unit_entry would have it's own unique sensor manifest.  
     *
     * DEPRECATED
     *
     */
    pub fn default_sensors(chemistry_manifest: &ChemistryManifest) -> Vec<SensorDefinition> {
        panic!("aoeu");
        vec![]
    }

    pub fn standard_sensors(chemistry_manifest: &ChemistryManifest) -> Vec<SensorDefinition> {
        let mut sensors = vec![];

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

        sensors
    }

    pub fn sensor_id_from_key<T: AsRef<str>>(&self, _key: T) -> SensorId {
        self.identify_sensor_from_key(_key).unwrap().id
    }

    pub fn identify_sensor_from_key<T: AsRef<str>>(&self, _key: T) -> Option<SensorDefinition> {
        let mut key = _key
            .as_ref()
            .clone()
            .trim_end_matches('"')
            .trim_start_matches('"')
            .to_string();
        //println!("identify_sensor_from_key: {}", &key);

        let original_key = key.clone();
        if !key.ends_with(")") {
            key = format!("{}{}", &key, "(0, 0)")
        }

        for (i, sensor) in self.sensors.iter().enumerate() {
            //println!("{}, {}", &sensor.key, &sensor.prop_key);

            //if key == &sensor.key || key == &sensor.prop_key { return Some(sensor.clone()) }
            if &key == &sensor.key || &original_key == &sensor.key {
                return Some(sensor.clone());
            }
        }

        return None;
    }
}

pub fn sensor_local_offsets(distance: i32) -> Vec<SensorCoordOffset> {
    let mut coords: Vec<SensorCoordOffset> = vec![];

    if distance == 0 {
        return vec![(0, 0)];
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
pub struct LocalPropertySensorManifest {
    entries: Vec<LocalPropertySensorEntry>,
}

impl LocalPropertySensorManifest {
    pub fn from_all_props(all_properties: &[Property]) -> Self {
        Self {
            entries: all_properties
                .iter()
                .enumerate()
                .map(|(i, prop)| LocalPropertySensorEntry {
                    long_key: prop.long_key.clone(),
                    property_id: prop.property_id.clone(),
                    property_offset_idx: prop.id,
                    distance: 1,
                })
                .collect::<Vec<_>>(),
        }
    }
    pub fn from_blacklist(blacklisted_prop_keys: &[String], all_properties: &[Property]) -> Self {
        Self {
            entries: all_properties
                .iter()
                .enumerate()
                .filter(|(i, prop)| !blacklisted_prop_keys.contains(&prop.long_key))
                .map(|(i, prop)| LocalPropertySensorEntry {
                    long_key: prop.long_key.clone(),
                    property_id: prop.property_id.clone(),
                    property_offset_idx: prop.id,
                    distance: 1,
                })
                .collect::<Vec<_>>(),
        }
    }
    pub fn from_whitelist(
        whitelisted_prop_keys: &[(String, usize)], //(key, distance)
        all_properties: &[Property],
    ) -> Self {
        let entries = whitelisted_prop_keys
            .iter()
            .map(|pair| {
                let prop = all_properties
                    .iter()
                    .find(|prop| prop.long_key == pair.0)
                    .unwrap();
                let distance = pair.1;

                LocalPropertySensorEntry {
                    long_key: prop.long_key.clone(),
                    property_id: prop.property_id.clone(),
                    property_offset_idx: prop.id,
                    distance: pair.1 as u8,
                }
            })
            .collect::<Vec<_>>();

        Self { entries }
    }
}

pub struct LocalPropertySensorEntry {
    pub long_key: String,
    pub property_id: PropertyId,
    pub property_offset_idx: usize,
    pub distance: u8,
}

pub mod tests {
    use super::*;

    #[test]
    fn test_sensor_local_coords() {
        assert_eq!(sensor_local_offsets(0), vec![(0, 0)]);
        assert_eq!(
            sensor_local_offsets(1),
            vec![
                (0, 0),
                (0, 1),
                (1, 1),
                (1, 0),
                (1, -1),
                (0, -1),
                (-1, -1),
                (-1, 0),
                (-1, 1)
            ]
        );
        assert_eq!(
            sensor_local_offsets(2),
            vec![
                (0, 0),
                (0, 1),
                (1, 1),
                (1, 0),
                (1, -1),
                (0, -1),
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (-1, 2),
                (0, 2),
                (1, 2),
                (2, 2),
                (2, 1),
                (2, 0),
                (2, -1),
                (2, -2),
                (1, -2),
                (0, -2),
                (-1, -2),
                (-2, -2),
                (-2, -1),
                (-2, 0),
                (-2, 1),
                (-2, 2)
            ]
        );
    }
}
