use crate::biology::genome::framed::common::*;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::*;

use crate::biology::genome::framed::builders::*;

pub fn genome1(gm: &GeneticManifest) -> Rc<CompiledFramedGenome> {
    let framed_vals = frame_from_single_channel(vec![gene(
        if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
        then_do!(pull_lever, 1),
    )])
    .build(&gm);

    FramedGenomeCompiler::compile(framed_vals, &gm)
}
