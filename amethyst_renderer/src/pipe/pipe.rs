use fnv::FnvHashMap as HashMap;
use hetseq::*;

use amethyst_core::specs::prelude::SystemData;

use crate::{
    error::{Error, Result},
    types::{Encoder, Factory},
};

use super::{stage::*, target::*};

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
pub trait PolyStages: for<'a> StagesData<'a> {
    ///
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &mut Encoder,
        factory: Factory,
        data: <Self as StagesData<'b>>::Data,
    );

    /// Distributes new targets
    fn new_targets(&mut self, new_targets: &HashMap<String, Target>);
}

impl<'a, HS> StagesData<'a> for List<(HS, List<()>)>
where
    HS: PolyStage,
{
    type Data = <HS as StageData<'a>>::Data;
}

impl<HS> PolyStages for List<(HS, List<()>)>
where
    HS: PolyStage,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &mut Encoder,
        factory: Factory,
        hd: <HS as StageData<'b>>::Data,
    ) {
        let List((ref mut hs, _)) = *self;
        hs.apply(encoders, factory, hd);
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

impl<HS, TS> PolyStages for List<(HS, TS)>
where
    HS: PolyStage,
    TS: PolyStages,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &mut Encoder,
        factory: Factory,
        (hd, td): <Self as StagesData<'b>>::Data,
    ) {
        let List((ref mut hs, ref mut ts)) = *self;
        hs.apply(encoders, factory.clone(), hd);
        ts.apply(encoders, factory, td);
    }

    fn new_targets(&mut self, new_targets: &HashMap<String, Target>) {
        let List((ref mut hs, ref mut ts)) = *self;
        HS::new_targets(hs, new_targets);
        TS::new_targets(ts, new_targets);
    }
}

/// The data requested from the `specs::World` by the Pipeline.
pub trait PipelineData<'a> {
    /// The data itself
    type Data: SystemData<'a> + Send;
}

/// Trait used for the pipeline.
pub trait PolyPipeline: for<'a> PipelineData<'a> {
    /// Retuns `ParallelIterator` which apply data to all stages
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        data: <Self as PipelineData<'b>>::Data,
    );

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

impl<L> PolyPipeline for Pipeline<L>
where
    L: PolyStages,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &mut Encoder,
        factory: Factory,
        data: <L as StagesData<'b>>::Data,
    ) {
        self.stages.apply(encoders, factory, data);
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
        Default::default()
    }
}

impl Default for PipelineBuilder<Queue<()>> {
    fn default() -> Self {
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
    fn build(self, fac: &mut Factory, out: &Target, multisampling: u16) -> Result<Self::Pipeline>;
}

impl<L, Z, R, Q> PipelineBuild for PipelineBuilder<Q>
where
    Q: IntoList<List = L>,
    L: for<'a> Functor<BuildStage<'a>, Output = Z>,
    Z: Try<Error, Ok = R>,
    R: PolyStages,
{
    type Pipeline = Pipeline<R>;
    fn build(mut self, fac: &mut Factory, out: &Target, multisampling: u16) -> Result<Pipeline<R>> {
        let mut targets = self
            .targets
            .drain(..)
            .map(|tb| tb.build(fac, out.size()))
            .collect::<Result<Targets>>()?;

        targets.insert("".into(), out.clone());

        // TODO: Remove this attribute when rustfmt plays nice.
        #[rustfmt::skip] // try is a reserved keyword in Rust 2018, must preserve keyword escape.
        let stages = self
            .stages
            .into_list()
            .fmap(BuildStage::new(fac, &targets, multisampling))
            .r#try()?;

        Ok(Pipeline { stages, targets })
    }
}

pub struct BuildStage<'a> {
    factory: &'a mut Factory,
    targets: &'a Targets,
    multisampling: u16,
}

impl<'a, 'b> BuildStage<'a> {
    fn new(factory: &'a mut Factory, targets: &'a Targets, multisampling: u16) -> Self {
        BuildStage {
            factory,
            targets,
            multisampling,
        }
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
        stage.build(self.factory, self.targets, self.multisampling)
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
        stage.build(self.factory, self.targets, self.multisampling)
    }
}
