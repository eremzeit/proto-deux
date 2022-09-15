use std::cell::Cell;

use super::common::{Frame, Gene, NUM_CHANNELS};

#[derive(Clone, Debug)]
pub struct FramedGenomeExecutionStats {
    pub frames: Vec<FrameExecutionStats>,
    pub eval_count: Cell<usize>,
}

impl FramedGenomeExecutionStats {
    pub fn empty() -> Self {
        Self {
            eval_count: Cell::new(0),
            frames: vec![],
        }
    }

    pub fn new(frames: &Vec<Frame>) -> Self {
        let mut s = Self {
            eval_count: Cell::new(0),
            frames: vec![],
        };

        s.initialize(frames);

        s
    }

    pub fn mark_eval(&self) {
        self.eval_count.set(self.eval_count.get() + 1);
    }

    pub fn initialize(&mut self, frames: &Vec<Frame>) {
        if self.frames.len() != 0 {
            return; //asume we're already prewarmed
        }

        while self.frames.len() < frames.len() {
            self.frames.push(FrameExecutionStats::new());
        }

        for (frame_idx, frame) in frames.iter().enumerate() {
            for (channel_idx, channel) in frames[frame_idx].channels.iter().enumerate() {
                self.initialize_channel(frame_idx, channel_idx, channel);
            }
        }
    }

    pub fn initialize_channel(
        &mut self,
        frame_idx: usize,
        channel_idx: usize,
        channel: &Vec<Gene>,
    ) {
        let mut channel_stats = &mut self.frames[frame_idx].channels[channel_idx];

        // while channel_stats.genes.len() < channel.len() {
        //     channel_stats.genes.push(GeneExecutionStats::new());
        // }

        for (gene_idx, gene) in channel.iter().enumerate() {
            channel_stats.genes.push(GeneExecutionStats::new());
            let mut gene_stats = channel_stats.genes.get_mut(gene_idx).unwrap();
            Self::initialize_gene(gene_stats, gene);
        }
    }

    pub fn initialize_gene(gene_stats: &mut GeneExecutionStats, gene: &Gene) {
        // while gene_stats.conditional.conjunctive_clauses.len()
        //     < gene.conditional.conjunctive_clauses.len()
        // {
        //     gene_stats
        //         .conditional
        //         .conjunctive_clauses
        //         .push(ConjunctiveClauseStats::new())
        // }

        for (conjuntive_i, conjunctive) in gene.conditional.conjunctive_clauses.iter().enumerate() {
            let mut conjunctive_clause = ConjunctionExpressionStats::new();

            for (bool_cond_idx, bool_cond) in conjunctive.boolean_variables.iter().enumerate() {
                conjunctive_clause
                    .bool_conditionals
                    .push(BooleanVariableStats::new())
            }

            gene_stats
                .disjunction_expression
                .conjunctive_expressions
                .push(conjunctive_clause);
        }
    }

    /**
     * A helper for incrementing the eval_true count for the frame, channel, and gene all at once.
     */
    pub fn mark_gene_and_parents_eval_true(
        &self,
        frame_idx: usize,
        channel_idx: usize,
        gene_idx: usize,
    ) {
        self.frames[frame_idx].mark_eval_true();
        self.frames[frame_idx].channels[channel_idx].mark_eval_true();
        self.frames[frame_idx].channels[channel_idx].genes[gene_idx].mark_eval_true();
    }
}

#[derive(Debug, Clone)]
pub struct FrameExecutionStats {
    pub eval_count: Cell<usize>,
    pub eval_true_count: Cell<usize>,
    pub channels: [ChannelExecutionStats; NUM_CHANNELS],
}

impl FrameExecutionStats {
    pub fn new() -> Self {
        Self {
            eval_count: Cell::new(0),
            eval_true_count: Cell::new(0),
            channels: [
                ChannelExecutionStats::new(),
                ChannelExecutionStats::new(),
                ChannelExecutionStats::new(),
                ChannelExecutionStats::new(),
            ],
        }
    }

    pub fn mark_eval(&self) {
        self.eval_count.set(self.eval_count.get() + 1);
    }
    pub fn mark_eval_true(&self) {
        self.eval_true_count.set(self.eval_true_count.get() + 1);
    }

    pub fn pct_true(&self) -> f32 {
        self.eval_true_count.get() as f32 / self.eval_count.get() as f32
    }
}

#[derive(Debug, Clone)]
pub struct ChannelExecutionStats {
    pub eval_count: Cell<usize>,
    pub eval_true_count: Cell<usize>,
    pub genes: Vec<GeneExecutionStats>,
}

impl ChannelExecutionStats {
    pub fn new() -> Self {
        Self {
            eval_count: Cell::new(0),
            eval_true_count: Cell::new(0),
            genes: vec![],
        }
    }
    pub fn mark_eval(&self) {
        self.eval_count.set(self.eval_count.get() + 1);
    }

    pub fn mark_eval_true(&self) {
        self.eval_true_count.set(self.eval_true_count.get() + 1);
    }

    pub fn pct_true(&self) -> f32 {
        self.eval_true_count.get() as f32 / self.eval_count.get() as f32
    }
}

#[derive(Debug, Clone)]
pub struct GeneExecutionStats {
    pub eval_count: Cell<usize>,
    pub eval_true_count: Cell<usize>,

    pub disjunction_expression: DisjunctionExpressionStats,
}

impl GeneExecutionStats {
    pub fn new() -> Self {
        Self {
            eval_count: Cell::new(0),
            eval_true_count: Cell::new(0),
            disjunction_expression: DisjunctionExpressionStats::new(),
        }
    }
    pub fn mark_eval(&self) {
        self.eval_count.set(self.eval_count.get() + 1);
    }
    pub fn mark_eval_true(&self) {
        self.eval_true_count.set(self.eval_true_count.get() + 1);
    }

    pub fn pct_true(&self) -> f32 {
        self.eval_true_count.get() as f32 / self.eval_count.get() as f32
    }
}

#[derive(Debug, Clone)]
pub struct DisjunctionExpressionStats {
    pub eval_count: Cell<usize>, // tracked on the gene stat object
    pub eval_true_count: Cell<usize>,
    pub conjunctive_expressions: Vec<ConjunctionExpressionStats>,
}

impl DisjunctionExpressionStats {
    pub fn new() -> Self {
        Self {
            eval_count: Cell::new(0),
            eval_true_count: Cell::new(0),
            conjunctive_expressions: vec![],
        }
    }
    pub fn mark_eval(&self) {
        self.eval_count.set(self.eval_count.get() + 1);
    }

    pub fn mark_eval_true(&self) {
        self.eval_true_count.set(self.eval_true_count.get() + 1);
    }

    pub fn pct_true(&self) -> f32 {
        self.eval_true_count.get() as f32 / self.eval_count.get() as f32
    }
}

#[derive(Debug, Clone)]
pub struct ConjunctionExpressionStats {
    pub eval_count: Cell<usize>,
    pub eval_true_count: Cell<usize>,

    pub bool_conditionals: Vec<BooleanVariableStats>,
}

impl ConjunctionExpressionStats {
    pub fn new() -> Self {
        Self {
            eval_count: Cell::new(0),
            eval_true_count: Cell::new(0),
            bool_conditionals: vec![],
        }
    }

    pub fn mark_eval(&self) {
        self.eval_count.set(self.eval_count.get() + 1);
    }

    pub fn mark_eval_true(&self) {
        self.eval_true_count.set(self.eval_true_count.get() + 1);
    }

    pub fn pct_true(&self) -> f32 {
        self.eval_true_count.get() as f32 / self.eval_count.get() as f32
    }
}

#[derive(Debug, Clone)]
pub struct BooleanVariableStats {
    pub eval_count: Cell<usize>,
    pub eval_true_count: Cell<usize>,
}

impl BooleanVariableStats {
    pub fn new() -> Self {
        Self {
            eval_count: Cell::new(0),
            eval_true_count: Cell::new(0),
        }
    }

    pub fn mark_eval(&self) {
        self.eval_count.set(self.eval_count.get() + 1);
    }

    pub fn mark_eval_true(&self) {
        self.eval_true_count.set(self.eval_true_count.get() + 1);
    }

    pub fn pct_true(&self) -> f32 {
        self.eval_true_count.get() as f32 / self.eval_count.get() as f32
    }
}
