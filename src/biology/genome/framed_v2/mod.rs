/*
    question: what is the intended lifetime of this construct?
        - for a single simulation, across all units?
        - for a single simulation, for each individual unit? (no)
            - unscaleable
        - across multiple simulations, where each sim is the same sim specs?

    given this question, it becomes clear that it should be storing stats
    that are useful if the number of genome executions is unknown.  We could
    store the raw counts that each node is executed, but we'd need to also
    keep a count

    So we need to be storing relative stats.  ie how often does this conditional
    get evaluated (which btw, only is useful on a per-conditional basis if we have
    early termination in conditionals)?  how often does it evaluated to true?

    To guide us we might first brainstorm how we'd be using these stats to guide
    our alterations.
    * on a gene basis only
        - genes are executed linearly so this simplifies the alteration logic
        - for each gene, we'd have a
            - percent chance of whether it gets evaluated at all in a single genome execution
                - we could avoid making alterations to genome ranges that never even get executed
                - though essentially we'd be saying ignore junk DNA
            - percent chance of whether it gets evaluated to true
                - focus on altering nodes that get evaluated to true too often or too seldom.
                - ie. there's an optimal "entropy" that we would shoot for. that optimal value would depend on how
                    long of a genome we are targetting, which could be affected by the genome execution cost
    * on a per-conditional basis
        - in the end we'd have a tree graph
            - ie. like a flame graph, almost like profiling data
        - using this to guide alterations would be a bit more complicated
            - it's unclear how we'd use this to guide alterations that affect a segment of the genome that is greater than 1 length


    possible framework for guiding alterations:
    * non-frame-shifting alterations
        - excludes those that...
            - change the frame length
            - change the gene length
            - affect multiple channels at once
        - frame shifting alterations:
            - catastraphically change the interpretation of that node plus all uncle nodes that are to the right of that node in the raw genome.

    question: are genes aligned across different channels?
        ie. if i skip ahead 2 genes and change to the next channel, is that
        the same as changing to the next channel and skipping ahead 2 genes?


    also, for each node in the genome, we need to track which addresses in
    the raw numerical genome that the node refers to (ie. a range of addresses).  And
    as such we might as well store the raw genome in this data structure as well
    becaus otherwise those address ranges don't have meaning.


*/

// use super::framed::builders::{CompiledFramedGenome, Frame};

// /**
//  * The parsed version of a genome that can be executed.
//  */
// pub struct ExecutableFramedGenome {
//     frames: FramedGenome,
//     execution_stats: usize,
// }

use super::framed::common::{CompiledFramedGenome, Frame};

pub struct FramedGenomeWithStats {
    pub frames: Vec<Frame>,
}

/**
 * The parsed version of a genome that can be executed.  
 */
pub struct FramedGenomeWithContext {
    frames: CompiledFramedGenome,
    execution_stats: usize,
}

pub enum Conditional {
    Or {
        is_negated: bool,
        children: Vec<Conditional>,
    },
}

// #[derive(Debug, Clone)]
// pub struct Frame {
//     pub channels: [Vec<Gene>; NUM_CHANNELS],
//     pub default_channel: u8,
// }

// #[derive(Debug, Clone)]
// pub struct Gene {
//     pub conditional: DisjunctiveClause,
//     pub operation: GeneOperationCall,
// }

// pub type DisjunctiveClause = (IsNegatedBool, Vec<ConjunctiveClause>);
// pub type ConjunctiveClause = (IsNegatedBool, Vec<BooleanVariable>);
