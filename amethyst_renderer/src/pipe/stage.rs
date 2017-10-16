//! A stage in the rendering pipeline.

use hetseq::*;

use error::{Error, Result};
use pipe::{Target, Targets};
use pipe::pass::{CompiledPass, Pass, PassApply, PassData};
use fnv::FnvHashMap as HashMap;
use rayon::iter::{Chain, ParallelIterator};
use specs::SystemData;

use types::{Encoder, Factory};

/// A stage in the rendering pipeline.
#[derive(Clone, Debug)]
pub struct Stage<L> {
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    enabled: bool,
    passes: L,
    target_name: String,
    target: Target,
}

impl Stage<List<()>> {
    /// Builds a new `PolyStage` which outputs to the `Target` with the given name.
    pub fn with_target<N: Into<String>>(target_name: N) -> StageBuilder<Queue<()>> {
        StageBuilder::new(target_name.into())
    }

    /// Builds a new `PolyStage` which outputs straight into the backbuffer.
    pub fn with_backbuffer() -> StageBuilder<Queue<()>> {
        StageBuilder::new("")
    }
}

impl<L> Stage<L> {
    /// Enables the `PolyStage` so it will execute on every frame.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables the `PolyStage`, preventing it from being executed on every frame.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Returns whether this `PolyStage` is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

pub trait PassesData<'a> {
    type Data: SystemData<'a> + Send;
}

pub trait PassesApply<'a> {
    type Apply: ParallelIterator<Item = ()>;
}

pub trait Passes: for<'a> PassesApply<'a> + for<'a> PassesData<'a> + Send + Sync {
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        data: <Self as PassesData<'b>>::Data,
    ) -> <Self as PassesApply<'a>>::Apply;

    /// Distributes new targets
    fn new_target(&mut self, new_target: &Target);
}

impl<'a, HP> PassesData<'a> for List<(CompiledPass<HP>, List<()>)>
where
    HP: Pass,
{
    type Data = <HP as PassData<'a>>::Data;
}

impl<'a, HP> PassesApply<'a> for List<(CompiledPass<HP>, List<()>)>
where
    HP: Pass,
{
    type Apply = <HP as PassApply<'a>>::Apply;
}
impl<HP> Passes for List<(CompiledPass<HP>, List<()>)>
where
    HP: Pass,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        hd: <HP as PassData<'b>>::Data,
    ) -> <HP as PassApply<'a>>::Apply {
        let (encoders, _) = encoders.split_at_mut(jobs_count);
        let List((ref mut hp, _)) = *self;
        hp.apply(encoders, hd)
    }

    fn new_target(&mut self, new_target: &Target) {
        let List((ref mut hp, _)) = *self;
        hp.new_target(new_target);
    }
}

impl<'a, HP, TP> PassesData<'a> for List<(CompiledPass<HP>, TP)>
where
    HP: Pass,
    TP: Passes,
{
    type Data = (<HP as PassData<'a>>::Data, <TP as PassesData<'a>>::Data);
}

impl<'a, HP, TP> PassesApply<'a> for List<(CompiledPass<HP>, TP)>
where
    HP: Pass,
    TP: Passes,
{
    type Apply = Chain<<HP as PassApply<'a>>::Apply, <TP as PassesApply<'a>>::Apply>;
}

impl<HP, TP> Passes for List<(CompiledPass<HP>, TP)>
where
    HP: Pass,
    TP: Passes,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        (hd, td): (<HP as PassData<'b>>::Data, <TP as PassesData<'b>>::Data),
    ) -> Chain<<HP as PassApply<'a>>::Apply, <TP as PassesApply<'a>>::Apply> {
        let (encoders, rest) = encoders.split_at_mut(jobs_count);
        let List((ref mut hp, ref mut tp)) = *self;
        hp.apply(encoders, hd).chain(tp.apply(rest, jobs_count, td))
    }

    fn new_target(&mut self, new_target: &Target) {
        let List((ref mut hp, ref mut tp)) = *self;
        hp.new_target(new_target);
        tp.new_target(new_target);
    }
}

///
pub trait StageData<'a> {
    type Data: SystemData<'a> + Send;
}

///
pub trait StageApply<'a> {
    type Apply: ParallelIterator<Item = ()>;
}

///
pub trait PolyStage
    : for<'a> StageApply<'a> + for<'a> StageData<'a> + Send + Sync {
    ///
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        data: <Self as StageData<'b>>::Data,
    ) -> <Self as StageApply<'a>>::Apply;
    /// Get number of encoders needed for this stage.
    fn encoders_required(jobs_count: usize) -> usize;

    /// Distributes new targets
    fn new_targets(&mut self, new_targets: &HashMap<String, Target>);
}

impl<'a, L> StageData<'a> for Stage<L>
where
    L: Passes,
{
    type Data = <L as PassesData<'a>>::Data;
}

impl<'a, L> StageApply<'a> for Stage<L>
where
    L: Passes + Length,
{
    type Apply = <L as PassesApply<'a>>::Apply;
}
impl<L> PolyStage for Stage<L>
where
    L: Passes + Length,
{
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoders: &'a mut [Encoder],
        jobs_count: usize,
        data: <L as PassesData<'b>>::Data,
    ) -> <Self as StageApply<'a>>::Apply {
        self.clear_color
            .map(|c| self.target.clear_color(&mut encoders[0], c));
        self.clear_depth
            .map(|d| self.target.clear_depth_stencil(&mut encoders[0], d));

        assert_eq!(Self::encoders_required(jobs_count), encoders.len());

        self.passes.apply(encoders, jobs_count, data)
    }

    #[inline]
    fn encoders_required(jobs_count: usize) -> usize {
        use std::cmp;
        cmp::max(L::len() * jobs_count, 1)
    }

    fn new_targets(&mut self, new_targets: &HashMap<String, Target>) {
        match new_targets.get(&self.target_name) {
            Some(target) => {
                self.target = target.clone();
                self.passes.new_target(target);
            }
            None => {
                eprintln!("Target name {:?} not found!", self.target_name);
            }
        }
    }
}

/// Constructs a new rendering stage.
#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub struct StageBuilder<Q> {
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    enabled: bool,
    passes: Q,
    target_name: String,
}

impl StageBuilder<Queue<()>> {
    /// Creates a new `StageBuilder` using the given target.
    pub fn new<T: Into<String>>(target_name: T) -> Self {
        StageBuilder {
            clear_color: None,
            clear_depth: None,
            enabled: true,
            passes: Queue::new(),
            target_name: target_name.into(),
        }
    }
}

impl<Q> StageBuilder<Q> {
    /// Clears the stage's target.
    pub fn clear_target<R, C, D>(mut self, color_val: C, depth_val: D) -> Self
    where
        R: Into<[f32; 4]>,
        C: Into<Option<R>>,
        D: Into<Option<f32>>,
    {
        self.clear_color = color_val.into().map(|c| c.into());
        self.clear_depth = depth_val.into();
        self
    }

    /// Sets whether the `PolyStage` is turned on by default.
    pub fn enabled(mut self, val: bool) -> Self {
        self.enabled = val;
        self
    }

    pub(crate) fn build<'a, L, Z, R>(
        self,
        fac: &'a mut Factory,
        targets: &'a Targets,
    ) -> Result<Stage<R>>
    where
        Q: IntoList<List = L>,
        L: for<'b> Functor<CompilePass<'b>, Output = Z>,
        Z: Try<Error, Ok = R>,
        R: Passes,
    {
        let out = targets
            .get(&self.target_name)
            .cloned()
            .ok_or(Error::NoSuchTarget(self.target_name.clone()))?;

        let passes = self.passes
            .into_list()
            .fmap(CompilePass::new(fac, &out))
            .try()?;

        Ok(Stage {
            clear_color: self.clear_color,
            clear_depth: self.clear_depth,
            enabled: self.enabled,
            passes,
            target: out,
            target_name: self.target_name,
        })
    }
}

impl<Q> StageBuilder<Queue<Q>> {
    /// Appends another `Pass` to the stage.
    pub fn with_pass<P: Pass>(self, pass: P) -> StageBuilder<Queue<(Queue<Q>, P)>> {
        StageBuilder {
            clear_color: self.clear_color,
            clear_depth: self.clear_depth,
            enabled: self.enabled,
            passes: self.passes.push(pass),
            target_name: self.target_name,
        }
    }
}



pub struct CompilePass<'a> {
    factory: &'a mut Factory,
    target: &'a Target,
}

impl<'a> CompilePass<'a> {
    fn new(factory: &'a mut Factory, target: &'a Target) -> Self {
        CompilePass { factory, target }
    }
}

impl<'a, P> HetFnOnce<(P,)> for CompilePass<'a>
where
    P: Pass,
{
    type Output = Result<CompiledPass<P>>;
    fn call_once(self, (pass,): (P,)) -> Result<CompiledPass<P>> {
        CompiledPass::compile(pass, self.factory, self.target)
    }
}
impl<'a, P> HetFnMut<(P,)> for CompilePass<'a>
where
    P: Pass,
{
    fn call_mut(&mut self, (pass,): (P,)) -> Result<CompiledPass<P>> {
        CompiledPass::compile(pass, self.factory, self.target)
    }
}
