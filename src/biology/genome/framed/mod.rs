#[macro_use]
pub mod util;

#[macro_use]
pub mod builders;

pub mod types;

//pub mod macros;

pub mod convert;
pub mod render;

pub mod parsing;

pub mod samples;

pub mod common {
	pub use biology::genome::framed::builders::*;
	pub use biology::genome::framed::convert::*;
	pub use biology::genome::framed::parsing::FramedGenomeParser;
	pub use biology::genome::framed::render::render_frames;
	pub use biology::genome::framed::types::*;
	pub use biology::genome::framed::util::identify_raw_param_string;
	pub use std::rc::Rc;
}

/* FramedGenome Execution Grammar
 *
 * genome := [gene1, gene2, gene3, ...]
 * gene := [predicate, operation],
 * predicate := [N_or_clauses, or_clause1, or_clause2, ... or_clauseN]
 * and_clause := [N_conditionals:u16:tiny_num, and_clause1, and_clause2, ...and_clause3]
 * conditional := [op:u16:small_num, param1:u16:small_num, param2:u16:small_num, param3:u16:small_num]  // 8 bytes
 * operation := [type:tiny_num, <reaction_or_metareaction>]  // 10 bytes
 * reaction := [reaction_call:u16:op_num, param1:u16:flex_num, param2:u16:flex_num, param3:u16:flex_num],  //8 bytes
 * meta := [meta_reaction_id:u16:op_num, param1:u16:flex_num, param2:u16_flex_num, param3:u16], //8 bytes
 *
 *
 * IF (foo AND bar) OR (baz AND qix) THEN X
 */

// how do i say, find an open square adjacent to me, and use that as a parameter?  is that what a register could be used for? ephemeral data?

/* FramedGenome Execution Grammar - v2?
 *
 * //frame := [register_preload, genome]
 * //register_load := [N_registers, sensor_idx]
 *
 * frame := [default_channel, genome]
 * genome := [gene1, gene2, gene3, ...]
 * gene := [predicate, operation],
 * disjunction := [N_or_clauses, IS_NEGATED, or_clause1, or_clause2, ... or_clauseN]
 * conjunction := [N_and_clauses, IS_NEGATED, conditional1, conditional2, ...conditionalN]
 * conditional := [operator_code, IS_NEGATED, param1_flags, param1, param1_flags, param2, param1_flags, param3]
 * op_param_flags := (<IS_SENSOR | IS_REGISTER | IS_LITERAL | IS_RANDOM | IS_SENSOR_CACHE>):byte,
 * operation := [op_type, reaction_or_metareaction]
 * op_param_flags := (<IS_SENSOR | IS_REGISTER | IS_LITERAL | IS_RANDOM | IS_SENSOR_CACHE>):byte,
 * reaction := [reaction_call_id, param1_flags, param1, param1_flags, param2, param1_flags, param3],
 * meta_reaction := [meta_reaction_id, param1_flags, param1, param1_flags, param2, param1_flags, param3],
 *
 */

// genome_frame!(
//    sensor_reference!('pos_res::cheese', 'aoeu'),
//    gene!(
//        if_any(
//            all(
//                (is_truthy, "pos_attr::is_cheese_source(0, 0)", 0, 0),
//                (gt, unit_res::cheese, 1000, 0)
//            )
//        ),
//
//        then_do(move_unit(literal(right)))
//    ),

//    gene!(''),
//    gene!('')
// )

//
//
// /*
//  *
//  * Frames:
//  * -maybe a frame could have a modifier bit at the beginning that causes a small-moderate transformation on the
//  *  frame's contents (maybe a subset of the space has its words reversed?)
//  * - maybe a frame can have a bit that enables a probabilistic execution of the conditionals in
//  *  that frame, as a way of supressing behavior?  and maybe this logic could be easily connected
//  *  to data in registers.
//  *
//  *
//  * ReactionDurations
//  * Should a unit have the option, during the course of a genome execution, of choosing to do a reaction multiple times?
//  * ie. the unit could say, I want to immediately execute this reaction 5 times, but then I have to
//  * skip my next 2 turns?  This is a way to incentivize some units to evolve towards doing multiple
//  * reactions, which would cut down the overhead for calculating that units genome.
//  *
//  *
//  * Meta Reactions
//  * jump to next/prev frame
//  * jump X frames ahead/behind
//  *
//  * should the genome be able to write to a register?
//  * registers 1-3
//  */
//
// /*
// *
// *
// * 8 bytes
// *
// * 1 bit = 8 bits = 4 * 2bits
// *
// * flex number  (flex_num)
// * 0,x,y,z -> x + y + z //12
// * 1,x,y,z -> 2x * 2y * 2z //
// * 2,x,y,z -> (x^3 * y^2 * z^1) //
// * 3,x,y,z -> (x^8 * y^2 * z^2) //65536
// *
// * big flex number (big_flex_num)  (2 words)
// * 0,x,y,z -> x + y + z //12
// * 1,x,y,z -> 2x * 2y * 2z //
// * 2,x,y,z -> (x^3 * y^2 * z^1) //
// * 3,x,y,z -> (x^8 * y^2 * z^2) //65536
// *
// * operation number (op_num)
// * 0,x,y,z -> x + y + z
// * 1,x,y,z -> x*y*z
// * 2,x,y,z -> 2x*y*z
// * 3,x,y,z -> 3x*y*z
//
// * small number (small_num)
// * 0,x,y,z -> x + y + z
// * 1,x,y,z -> x + y + z
// * 2,x,y,z -> x + y + z
// * 3,x,y,z -> 2x + 2y + 2z
//
// * tiny number (tiny_num, 0-3)
// * 0,x,y,z -> x
// * 1,x,y,z -> x
// * 2,x,y,z -> x
// * 3,x,y,z -> x
// *
// */
