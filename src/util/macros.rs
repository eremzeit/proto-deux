use crate::simulation::common::*;
use std::sync::Mutex;

#[macro_export]
macro_rules! wrap_chemistry {
    ($chem:expr) => {
        Box::new($chem)
    };
}

macro_rules! _define_struct {
    ($struct_name:ident, $idx_type:ident, [$([$attribute:ident,  $_: ty]),*]) => {
        pub struct $struct_name {
            $(
                pub $attribute: $idx_type,
            )*
        }
    };
    ($struct_name:ident, $idx_type:ident, [$([$attribute:ident, $_:expr]),*]) => {
        pub struct $struct_name {
            $(
                pub $attribute: $idx_type,
            )*
        }
    };
}

macro_rules! _define_impl__new {
    (ATTRIBUTE [$([$attribute:ident, $type_category:ident]),*]) => {
        pub fn new() -> Self {
            let mut x = 0;
            Self {
                $(
                    $attribute: { let _x = x; x += 1; _x },
                )*
            }
        }
    };

    (RESOURCE [$([$resource_name:ident, $is_streamed:stmt]),*]) => {
        pub fn new() -> Self {
            let mut x = 0;
            Self {
                $(
                    $resource_name: { let _x = x; x += 1; _x },
                )*
            }
        }
    };
}

macro_rules! _define_impl__defs_fn {
    ($def_type:ty, [$([$attribute:ident, $type_category:ident]),*]) => {
        pub fn make_defs() -> Vec<$def_type> {

           let mut items: Vec<$def_type> = vec![];

           let mut i = 0;

           $(
               items.push({
                   let name = stringify!($attribute).to_string();
                   let _type = AttributeDefinitionType::$type_category;
                   let _i = i;
                   i = i + 1;

                   <$def_type>::new(
                        &name,
                        _type,
                        _i,
                   )
               });
           )*

            items
        }
    };
}

macro_rules! _define_impl {
    ($struct_name:ident, $def_type:ty, $all:tt) => {
        impl $struct_name {
            _define_impl__new!(ATTRIBUTE $all);
            _define_impl__defs_fn!($def_type, $all);
        }
    }
}

macro_rules! _define_impl__res {
    ($struct_name:ident, $def_type:ty, $all:tt) => {
        impl $struct_name {
            _define_impl__new!(RESOURCE $all);
            _define_impl__defs_fn__res!($def_type, $all);
        }
    }
}

// (RESOURCE $([$resource_name:ident, $is_streamed:stmt]),*)

macro_rules! _define_impl__defs_fn__res {
    ($def_type:ty, [$([$attribute:ident, $is_streamed:expr]),*]) => {
        pub fn make_defs() -> Vec<$def_type> {

           let mut items: Vec<$def_type> = vec![];

           let mut i = 0;

           $(
               items.push({
                   let name = stringify!($attribute).to_string();
                   //let _type = Resource::$type_category;
                   let _i = i;
                   i = i + 1;

                   <$def_type>::new(
                        &name,
                        $is_streamed,
                        _i,
                   )
               });
           )*

            items
        }
    };
}

#[macro_export]
macro_rules! def_simulation_attributes {
    ($all:tt) => {
        _define_struct!(SimulationAttributesLookup, AttributeIndex, $all);
        _define_impl!(
            SimulationAttributesLookup,
            SimulationAttributeDefinition,
            $all
        );
    };
}

#[macro_export]
macro_rules! def_unit_entry_attributes {
    ($all:tt) => {
        _define_struct!(UnitEntryAttributesLookup, AttributeIndex, $all);
        _define_impl!(
            UnitEntryAttributesLookup,
            UnitEntryAttributeDefinition,
            $all
        );
    };
}

#[macro_export]
macro_rules! def_unit_attributes {
    ($all:tt) => {
        _define_struct!(UnitAttributesLookup, AttributeIndex, $all);
        _define_impl!(UnitAttributesLookup, UnitAttributeDefinition, $all);
    };
}

#[macro_export]
macro_rules! def_position_attributes {
    ($all:tt) => {
        _define_struct!(PositionAttributesLookup, AttributeIndex, $all);
        _define_impl!(PositionAttributesLookup, PositionAttributeDefinition, $all);
    };
}

#[macro_export]
macro_rules! def_unit_resources {
    ($all:tt) => {
        _define_struct!(UnitResourcesLookup, ResourceIndex, $all);
        _define_impl__res!(UnitResourcesLookup, UnitResourceDefinition, $all);
    };
}

#[macro_export]
macro_rules! def_position_resources {
    ($all:tt) => {
        _define_struct!(PositionResourcesLookup, ResourceIndex, $all);
        _define_impl__res!(PositionResourcesLookup, PositionResourceDefinition, $all);
    };
}

macro_rules! get_type_from_attribute_def {
    (Number) => {
        i32
    };
    (Boolean) => {
        bool
    };
}

/*
 *  Reactions
 */
#[macro_export]
macro_rules! reagent {
    ( $key:expr $(,$param:expr)* $(,)?) => {
        {
            {
                let mut params: Vec<ActionParam> = vec![];

                $(
                    params.push($param);
                )*

                ReagentDefinition::new($key, params)
            }
        }
    };
}

#[macro_export]
macro_rules! param_arg {
    ($x:ident) => {
        ActionParam::Placeholder(ActionParamType::$x)
    };
}

#[macro_export]
macro_rules! param_value {
    ($x:ident, $val:expr) => {
        ActionParam::$x($val)
    };
}

//reagent_value!(UnitResourceKey("cheese")),

//#[macro_export]
// macro_rules! def_reactions {
//     ([$x:block) => {
//         def_reactions__fn!($x);
//     }
// }

#[macro_export]
macro_rules! def_reactions {
    ($($reaction:expr),*, $(,)?) => {
        pub fn get_reactions() -> Vec<ReactionDefinition> {
            let mut reactions: Vec<ReactionDefinition> = vec![];

            $(
                reactions.push($reaction);
            )*

            reactions
        }
    }

}

#[macro_export]
macro_rules! reaction {
    ( $key:expr $(, $reagent:expr)*, $(,)?) => {
        {
            let mut reagents = vec![];

            $(
                reagents.push($reagent);
            )*

            ReactionDefinition::new($key, reagents)
        }
    };
}
#[macro_export]
macro_rules! assert_coords_valid_for_size {
    ( $coord:expr, $size:expr) => {{
        assert!(
            ($coord.0 < $size.0) && ($coord.1 < $size.1),
            "cannot get grid item at {:?} for a grid with size {:?}",
            $coord,
            size
        );
    }};
}
#[macro_export]
macro_rules! assert_coords_valid_for_world {
    ( $coord:expr, $world:expr) => {{
        assert!(
            ($coord.0 < $world.size.0) && ($coord.1 < $world.size.1),
            "cannot get grid item at {:?} for a grid with size {:?}",
            $coord,
            $world.size
        );
    }};
}

#[macro_export]
macro_rules! assert_unit_attribute_at {
    ( $sim:expr, $coord:expr, $key:expr, $amount:expr ) => {{
        let value = $sim
            .world
            .get_unit_attribute_at($coord, $sim.unit_attr_id_by_key($key));
        assert_eq!(value, $amount);
    }};
}

#[macro_export]
macro_rules! assert_position_attribute_at {
    ( $sim:expr, $coord:expr, $key:expr, $amount:expr ) => {{
        let value = $sim
            .world
            .get_unit_attribute_at($coord, $sim.unit_attr_id_by_key($key));
        assert_eq!(value, $amount);
    }};
}

#[macro_export]
macro_rules! assert_unit_resource_at {
    ( $sim:expr, $coord:expr, $key:expr, $amount:expr ) => {{
        let amt: UnitResourceAmount = $amount;
        let value = $sim
            .world
            .get_unit_resource_at($coord, $sim.unit_resource_id_by_key($key));
        assert_eq!(value, $amount);
    }};
}

#[macro_export]
macro_rules! assert_unit_at {
    ( $sim:expr, $coord:expr ) => {{
        assert!(
            $sim.world.get_unit_at($coord).is_some(),
            "Expected used to exist at {:?}, ",
            $coord
        );
    }};
}
