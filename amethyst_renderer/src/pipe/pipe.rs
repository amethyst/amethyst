
use hetseq::*;
use rayon::iter::{Chain, ParallelIterator};
use specs::SystemData;

use super::stage::*;
use super::target::*;

// use color::Rgba;
use error::{Error, Result};
use fnv::FnvHashMap as HashMap;
use types::{Encoder, Factory};


/// Defines how the rendering pipeline should be configured.
#[derive(Clone, Debug)]
pub struct Pipeline<L> {
    stages: L,
    targets: HashMap<String, Target>,
}

impl Pipeline<List<()>> {
    /// Builds a new renderer pipeline.
    pub fn build() -> PipelineBuilder<Queue<()>> {
        PipelineBuilder::new()
    }
}

impl<L> Pipeline<L> {
    /// Returns an immutable reference to all targets and their name strings.
    pub fn targets(&self) -> &HashMap<String, Target> {
        &self.targets
    }
}

///
pub trait StagesData<'a> {
    ///
    type Data: SystemData<'a> + Send;
}

///
pub trait StagesApply<'a> {
    ///
    type Apply: ParallelIterator<Item = ()>;
}

///
pub trait PolyStages
    : for<'a> StagesApply<'a> + for<'a> StagesData<'a> + Send + Sync {
    ///
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        data: <Self as StagesData<'b>>::Data,
    ) -> <Self as StagesApply<'a>>::Apply;
    ///
    fn encoders_required(jobs_count: usize) -> usize;

    /// Distributes new targets
    fn new_targets(&mut self, new_targets: &HashMap<String, Target>);
}

impl<'a, HS> StagesData<'a> for List<(HS, List<()>)>
where
    HS: PolyStage,
{
    type Data = <HS as StageData<'a>>::Data;
}

impl<'a, HS> StagesApply<'a> for List<(HS, List<()>)>
where
    HS: PolyStage,
{
    type Apply = <HS as StageApply<'a>>::Apply;
}

impl<HS> PolyStages for List<(HS, List<()>)>
where
    HS: PolyStage,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        hd: <HS as StageData<'b>>::Data,
    ) -> <HS as StageApply<'a>>::Apply {
        let (encoders, _) = encoders.split_at_mut(HS::encoders_required(jobs_count));
        let List((ref mut hs, _)) = *self;
        hs.apply(encoders, jobs_count, hd)
    }

    fn encoders_required(jobs_count: usize) -> usize {
        HS::encoders_required(jobs_count)
    }

    fn new_targets(&mut self, new_targets: &HashMap<String, Target>) {
        let List((ref mut hs, _)) = *self;
        HS::new_targets(hs, new_targets);
    }
}

impl<'a, HS, TS> StagesData<'a> for List<(HS, TS)>
where
    HS: PolyStage,
    TS: PolyStages,
{
    type Data = (<HS as StageData<'a>>::Data, <TS as StagesData<'a>>::Data);
}

impl<'a, HS, TS> StagesApply<'a> for List<(HS, TS)>
where
    HS: PolyStage,
    TS: PolyStages,
{
    type Apply = Chain<<HS as StageApply<'a>>::Apply, <TS as StagesApply<'a>>::Apply>;
}

impl<HS, TS> PolyStages for List<(HS, TS)>
where
    HS: PolyStage,
    TS: PolyStages,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        (hd, td): <Self as StagesData<'b>>::Data,
    ) -> <Self as StagesApply<'a>>::Apply {
        let (encoders, left) = encoders.split_at_mut(HS::encoders_required(jobs_count));
        let List((ref mut hs, ref mut ts)) = *self;
        hs.apply(encoders, jobs_count, hd)
            .chain(ts.apply(left, jobs_count, td))
    }

    fn encoders_required(jobs_count: usize) -> usize {
        HS::encoders_required(jobs_count) + TS::encoders_required(jobs_count)
    }

    fn new_targets(&mut self, new_targets: &HashMap<String, Target>) {
        let List((ref mut hs, ref mut ts)) = *self;
        HS::new_targets(hs, new_targets);
        TS::new_targets(ts, new_targets);
    }
}

///
pub trait PipelineData<'a> {
    ///
    type Data: SystemData<'a> + Send;
}

///
pub trait PipelineApply<'a> {
    ///
    type Apply: ParallelIterator<Item = ()>;
}

///
pub trait PolyPipeline
    : for<'a> PipelineApply<'a> + for<'a> PipelineData<'a> + Send + Sync {
    /// Retuns `ParallelIterator` which apply data to all stages
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &'a mut [Encoder],
        jobs_count: usize,
        data: <Self as PipelineData<'b>>::Data,
    ) -> <Self as PipelineApply<'a>>::Apply;

    /// Returns number of `Encoder`s required
    fn encoders_required(jobs_count: usize) -> usize;

    /// Resizes the pipeline targets
    fn new_targets(&mut self, new_targets: HashMap<String, Target>);

    /// Returns an immutable reference to all targets and their name strings.
    fn targets(&self) -> &HashMap<String, Target>;
}

impl<'a, L> PipelineData<'a> for Pipeline<L>
where
    L: PolyStages,
{
    type Data = <L as StagesData<'a>>::Data;
}

impl<'a, L> PipelineApply<'a> for Pipeline<L>
where
    L: PolyStages,
{
    type Apply = <L as StagesApply<'a>>::Apply;
}

impl<L> PolyPipeline for Pipeline<L>
where
    L: PolyStages,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        data: <L as StagesData<'b>>::Data,
    ) -> <L as StagesApply<'a>>::Apply {
        self.stages.apply(encoders, jobs_count, data)
    }

    fn encoders_required(jobs_count: usize) -> usize {
        L::encoders_required(jobs_count)
    }

    fn new_targets(&mut self, new_targets: HashMap<String, Target>) {
        self.stages.new_targets(&new_targets);
        self.targets = new_targets;
    }

    /// Returns an immutable reference to all targets and their name strings.
    fn targets(&self) -> &HashMap<String, Target> {
        self.targets()
    }
}

/// Constructs a new pipeline with the given render targets and layers.
#[derive(Clone, Debug)]
pub struct PipelineBuilder<Q> {
    stages: Q,
    targets: Vec<TargetBuilder>,
}

impl PipelineBuilder<Queue<()>> {
    /// Creates a new PipelineBuilder.
    pub fn new() -> Self {
        PipelineBuilder {
            stages: Queue::new(),
            targets: Vec::new(),
        }
    }
}

impl<Q> PipelineBuilder<Queue<Q>> {
    /// Constructs a new stage in this pipeline.
    pub fn with_stage<P>(
        self,
        sb: StageBuilder<P>,
    ) -> PipelineBuilder<Queue<(Queue<Q>, StageBuilder<P>)>> {
        PipelineBuilder {
            stages: self.stages.push(sb),
            targets: self.targets,
        }
    }
}


impl<Q> PipelineBuilder<Q> {
    /// Constructs a new render target for this pipeline.
    pub fn with_target(mut self, tb: TargetBuilder) -> Self {
        self.targets.push(tb);
        self
    }
}

///
pub trait PipelineBuild {
    /// Resuling pipeline
    type Pipeline: PolyPipeline;

    /// Build pipeline
    fn build(self, fac: &mut Factory, out: &Target) -> Result<Self::Pipeline>;
}

impl<L, Z, R, Q> PipelineBuild for PipelineBuilder<Q>
where
    Q: IntoList<List = L>,
    L: for<'a> Functor<BuildStage<'a>, Output = Z>,
    Z: Try<Error, Ok = R>,
    R: PolyStages,
{
    type Pipeline = Pipeline<R>;
    fn build(mut self, fac: &mut Factory, out: &Target) -> Result<Pipeline<R>> {
        let mut targets = self.targets
            .drain(..)
            .map(|tb| tb.build(fac, out.size()))
            .collect::<Result<Targets>>()?;

        targets.insert("".into(), out.clone());

        let stages = self.stages
            .into_list()
            .fmap(BuildStage::new(fac, &targets))
            .try()?;

        Ok(Pipeline { stages, targets })
    }
}

pub struct BuildStage<'a> {
    factory: &'a mut Factory,
    targets: &'a Targets,
}

impl<'a> BuildStage<'a> {
    fn new(factory: &'a mut Factory, targets: &'a Targets) -> Self {
        BuildStage { factory, targets }
    }
}

impl<'a, Q, L, Z, R> HetFnOnce<(StageBuilder<Q>,)> for BuildStage<'a>
where
    Q: IntoList<List = L>,
    L: for<'b> Functor<CompilePass<'b>, Output = Z>,
    Z: Try<Error, Ok = R>,
    R: Passes,
{
    type Output = Result<Stage<R>>;
    fn call_once(self, (stage,): (StageBuilder<Q>,)) -> Result<Stage<R>> {
        stage.build(self.factory, self.targets)
    }
}

impl<'a, Q, L, Z, R> HetFnMut<(StageBuilder<Q>,)> for BuildStage<'a>
where
    Q: IntoList<List = L>,
    L: for<'b> Functor<CompilePass<'b>, Output = Z>,
    Z: Try<Error, Ok = R>,
    R: Passes,
{
    fn call_mut(&mut self, (stage,): (StageBuilder<Q>,)) -> Result<Stage<R>> {
        stage.build(self.factory, self.targets)
    }
}
