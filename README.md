# Overview

Protomolecule is an general purpose engine for executing and evolving agents in a discreet 2D environment with arbitrary rulesets.

The goal?  I want to open-endedly evolve a diverse set of agents that demonstrate the qualities of "aliveness".

The codebase is in alpha and will probably change as I work out the special sauce of the evolutionary algorithm.

### Current status

- [x] Simulation execution
- [x] A few simple chemistries included for debugging (ie. cheese chemistry)
- [x] Genome compilation from binary data and rendering as readable text
- [x] Long-running evolutionary experiments that demonstrate it's possible to start with random genome data and evolve into basic solutions
- [x] Multithreaded execution
- [x] Rendering a realtime visual grid using OpenGL
- [ ] Features supporting cluster computing
- [ ] More advanced chemistries that support more interesting problem spaces for agents to solve
- [ ] More advanced tools and metrics used to detect and handle when evolution converges on "local maxima"



### Basic definitions

- A simulation is one instance of the 2D grid, along with a chemistry specifications.
- Each location in the grid is called a position. 
- Each instance of an agent is called a unit.
- A chemistry is a custom struct and implementation that implements the Chemistry trait, which among other things defines a list of existing...
  - UnitAttributes (eg can_move)
  - UnitResources (eg. cheese)
  - PositionAttributes (eg. is_cheese_source)
  - PositionResources (eg. cheese)
  - Reactions: these are the possible actions that a unit can execute to cause a change in the world (eg. move_unit, eat_food, make_new_unit) 
- Each reaction is a list of sub-actions (ie. "reagents", "actions") that execute but require parameters to be passed, either by the chemistry or the unit.  Each action corresponds to a function that implements that desired behavior.  
- Each position instance has its own set of PositionAttributes and PositionResources.
- Each unit instance has its own set of UnitAttributes and UnitResources.

### Example screenshot

![image](https://user-images.githubusercontent.com/1264533/188751481-42473ddd-b1df-4157-9d98-3ddf1b3eb278.png)

The above screenshot is a simple example using the very simple Cheese chemistry in which agents just move around, collect cheese, and reproduce.  In this rendering, red circles are the units.  Green squares means there's cheese.

## An example chemistry

A chemistry is a struct that implements the chemistry trait.  Among other duties, a chemistry defines what attributes and resources that positions and resources have available.  These definitions are made using macros to allow for compile-time definition of the world's rules, which helps with performance.


Here's an example of definitions from the cheese chemistry.
```
pub mod defs {
    def_simulation_attributes! {[
        [total_cheese_consumed, Number]
    ]}

    def_unit_attributes! {[
        [can_move, Number]
    ]}

    def_position_attributes! {[
        [is_cheese_source, Boolean],
        [is_air_source, Boolean]
    ]}

    def_position_resources! {[
        [cheese, false],
        [air, false]
    ]}

    def_unit_resources! {[
       [cheese, false],
       [air, true]
    ]}
    def_unit_entry_attributes! {[
        [total_cheese_consumed, Number]
    ]}
    
    def_reactions! {
        reaction!("gobble_cheese",
            reagent!("gobble_cheese"),
        ),

        reaction!("move_unit",  // move_unit means... subtract the movement cost and the move the unit
            reagent!("offset_unit_resource", // moving costs the unit resource cheese
                constant_arg!(UnitResourceKey, "cheese"),
                chemistry_arg!(UnitResourceAmount, move_cost), // this amount is configurable on a per-simulation basis
                constant_arg!(Boolean, false),
            ),
            reagent!("move_unit",
                unit_behavior_arg!(Direction)
            ),
        ),

        reaction!("new_unit",
            reagent!("offset_unit_resource",
                constant_arg!(UnitResourceKey, "cheese"),
                chemistry_arg!(UnitResourceAmount, new_unit_cost),
                constant_arg!(Boolean, false),
            ),
            reagent!("new_unit",
                unit_behavior_arg!(Direction),
            ),
        ),
    }
}
```


# Genome Execution 

Genomes currently are just arrays of unsigned 64-bit numbers.  The compilation of the genome is summarized by the following grammar.

```
GENOME := [frame1, frame2, ..., frameN]
FRAME := [frame_size, default_channel, gene1, ..., geneN]
GENE := [disjunction, operation]
DISJUNCTION (or clause) := [N_or_clauses, IS_NEGATED, clause1, clause2, ... clauseN]
CONJUNCTION (and clause) := [N_and_clauses, IS_NEGATED, conditional1, conditional2, ...conditionalN]
CONDITIONAL := [operator_code, IS_NEGATED, param1_flag, param1, param2_flag, param2, param3_flag, param3]
OP_PARAM_FLAG := PARAM_FLAG
OPERATION := [reaction_type_code, reaction] | [metareaction_type_code, meta_reaction]
(ie. an operation is either a reaction or meta-reaction, defined by the first value in the pair)

REACTION := [reaction_call_id, reaction_param_flag1, param1, reaction_param_flag2, param2, reaction_param_flag3, param3],
REACTION_PARAM_FLAG := PARAM_FLAG
(a reaction, when executed causes the genome execution to terminate, resulting in that reaction being performed)

META_REACTION := [meta_metareaction_id, metareaction_param_flag1, param1, metareaction_param_flag2, param2, reaction_param_flag3, param3],
METAREACTION_PARAM_FLAG := PARAM_FLAG
(a meta reaction when executed causes a change in the genome execution (eg. jump ahead a frame, change default channel, etc))

PARAM_FLAG := (<IS_SENSOR | IS_REGISTER | IS_LITERAL | IS_RANDOM | IS_SENSOR_CACHE>):byte,
(a param flag comes before a parameter value to express which semantics to use to evaluate that parameter value (eg. sensor value, constant, etc))
```

A genome, after compiling, can be rendered into a human readable form which might look something like:

```
CALL move_unit(Constant(1)) IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && unit_res::cheese(0, 0) > Constant(1000))
CALL gobble_cheese() IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && unit_res::cheese(0, 0) > Constant(300))
CALL gobble_cheese() IF (unit_res::cheese(0, 0) < Constant(60) && pos_res::cheese(0, 0) > Constant(20))
CALL move_unit(Random(4)) IF random_hundred < Constant(20)
CALL new_unit(Constant(0)) IF unit_res::cheese(0, 0) > Constant(600)
CALL gobble_cheese() IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && unit_res::cheese(0, 0) < Constant(1000))
CALL move_unit(Constant(0)) IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && is_truthy(pos_attr::is_cheese_source(0, 1)))
CALL move_unit(Constant(1)) IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && is_truthy(pos_attr::is_cheese_source(1, 0)))
CALL move_unit(Constant(3)) IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && is_truthy(pos_attr::is_cheese_source(-1, 0)))
CALL move_unit(Constant(2)) IF (is_truthy(pos_attr::is_cheese_source(0, 0)) && is_truthy(pos_attr::is_cheese_source(0, -1)))
```


# Genome Evolution

In evolution experiments, genomes are organized into gene pools of size N.  Each iteration of the experiment, some subset of the genomes are selected to compete in a single simulation.  Based on the fitness results of that simulation, the genome rank scores are adjusted.  Then some subset of genomes are selected to be eliminated and other genomes copy but are modified via some genome alteration (eg. point insertion, point deletion, crossover, random region insertion).  Then the next iteration begins.  Through the principle of survival of the fittest, the gene pool over time evolves towards increasing fitness scores.  


![image](https://user-images.githubusercontent.com/1264533/189597021-b7ee0e42-f89d-4787-a953-5cdc4e078d0e.png)
Here's an example of how fitness increases even over a relatively short number of iterations.  Notice how there are long periods of stagnation with sudden increases in fitness.


# Running 

### Some examples

To run a single simulation with some preset genomes inside a ui view

```
cargo run sim_ui -c cheese -s with_genome -F 50
```


To run a long-running evolutionary experiment using the `simple_cheese` configuration template
```
cargo run -r exp -s simple_cheese -n foo_cheese
```


To run the test suite: `cargo test`
