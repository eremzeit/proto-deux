pub struct FramedGenomeExecutionStats {
    pub frames: Vec<FrameExecutionStats>,
    pub eval_count: usize,
}

#[derive(Debug, Clone)]
pub struct FrameExecutionStats {
    pub eval_count: usize,
    pub eval_true_count: usize,
    pub genes: Vec<GeneExecutionStats>,
}

#[derive(Debug, Clone)]
pub struct GeneExecutionStats {
    pub eval_count: usize,
    pub eval_true_count: usize,

    pub conditional: DisjunctiveClauseStats,
}

#[derive(Debug, Clone)]
pub struct DisjunctiveClauseStats {
    pub eval_count: usize,
    pub eval_true_count: usize,

    pub conjunctive_clauses: Vec<DisjunctiveClauseStats>,
}

#[derive(Debug, Clone)]
pub struct ConjunctiveClauseStats {
    pub eval_count: usize,
    pub eval_true_count: usize,

    pub bool_conditional: Vec<BooleanConditionalStats>,
}

#[derive(Debug, Clone)]
pub struct BooleanConditionalStats {
    pub eval_count: usize,
    pub eval_true_count: usize,
}
