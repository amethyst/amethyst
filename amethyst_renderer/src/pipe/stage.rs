//! A stage in the rendering pipeline.

use error::{Error, Result};
use pipe::{Target, Targets};
use pipe::pass::{CompiledPass, Pass, Description};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, MapWith, ParallelIterator, Zip};
use rayon::iter::internal::UnindexedConsumer;
use rayon::slice::Chunks;
use rayon::vec::IntoIter;
use scene::{Model, Scene};
use types::{Encoder, Factory};

/// TODO: Eliminate all this explicit typing once `impl Trait` lands.
type ApplyPassFn<'a> = fn(&mut &'a CompiledPass, (&'a [Model], &'a mut Encoder)) -> (&'a CompiledPass, &'a [Model], &'a mut Encoder);
type Workload<'a> = Zip<Chunks<'a, Model>, IntoIter<&'a mut Encoder>>;

/// Parallel iterator of all pass.
pub(crate) struct DrawUpdate<'a> {
    inner: IntoIter<MapWith<Workload<'a>, &'a CompiledPass, ApplyPassFn<'a>>>,
}

impl<'a> ParallelIterator for DrawUpdate<'a> {
    type Item = (&'a CompiledPass, &'a [Model], &'a mut Encoder);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.flat_map(|i| i).drive_unindexed(consumer)
    }
}

/// A stage in the rendering pipeline.
#[derive(Clone, Debug)]
pub struct Stage {
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    enabled: bool,
    passes: Vec<CompiledPass>,
    target: Target,
}

impl Stage {
    /// Creates a new stage using the Target with the given name.
    pub fn with_target<N: Into<String>>(target_name: N) -> StageBuilder {
        StageBuilder::new(target_name.into())
    }

    /// Creates a new stage which draws straight into the backbuffer.
    pub fn with_backbuffer() -> StageBuilder {
        StageBuilder::new("")
    }

    /// Sets whether this layer should execute.
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Checks whether this layer is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Applies all passes within this stage.
    pub(crate) fn apply<'a, T: Iterator<Item = &'a mut Encoder>>(&'a self, mut encoders: T, scene: &'a Scene) -> DrawUpdate<'a> {
        use num_cpus;

        self.clear_color.map(|c| self.target.clear_color(encoders.nth(0).unwrap(), c));
        self.clear_depth.map(|d| self.target.clear_depth_stencil(encoders.nth(0).unwrap(), d));

        let mut update = Vec::new();
        for pass in self.passes.iter() {
            let mut models = scene.par_chunks_models(num_cpus::get());
            let enc: Vec<_> = encoders.by_ref().take(models.len()).collect();
            update.push(models.zip(enc).map_with(pass, (|pass, (models, enc)| (*pass, models, enc)) as ApplyPassFn<'a>));
        }

        DrawUpdate { inner: update.into_par_iter() }
    }

    /// Get count of parallelable passes
    pub fn encoders_required(&self, jobs_count: usize) -> usize {
        use std::cmp;
        self.passes.len() * (cmp::max(jobs_count, 1) - 1) + 1
    }
}

/// Constructs a new rendering stage.
#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub struct StageBuilder {
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    enabled: bool,
    #[derivative(Debug = "ignore")]
    passes: Vec<Description>,
    target_name: String,
}

impl StageBuilder {
    /// Creates a new `StageBuilder` using the given target.
    pub fn new<T: Into<String>>(target_name: T) -> Self {
        StageBuilder {
            clear_color: None,
            clear_depth: None,
            enabled: true,
            passes: Vec::new(),
            target_name: target_name.into(),
        }
    }

    /// Clears the stage's target.
    pub fn clear_target<R, C, D>(mut self, color_val: C, depth_val: D) -> Self
        where R: Into<[f32; 4]>,
              C: Into<Option<R>>,
              D: Into<Option<f32>>
    {
        self.clear_color = color_val.into().map(|c| c.into());
        self.clear_depth = depth_val.into();
        self
    }

    /// Appends another `Pass` to the stage.
    pub fn with_model_pass<P: Pass + 'static>(mut self, pass: P) -> Self {
        self.passes.push(Description::new(pass));
        self
    }

    /// Sets whether the `Stage` is turned on by default.
    pub fn enabled(mut self, val: bool) -> Self {
        self.enabled = val;
        self
    }

    /// Builds and returns the stage.
    pub(crate) fn build(mut self, fac: &mut Factory, targets: &Targets) -> Result<Stage> {
        let out = targets
            .get(&self.target_name)
            .cloned()
            .ok_or(Error::NoSuchTarget(self.target_name))?;

        let passes = self.passes
            .drain(..)
            .map(|pb| pb.compile(fac, &out))
            .collect::<Result<_>>()?;

        Ok(Stage {
               clear_color: self.clear_color,
               clear_depth: self.clear_depth,
               enabled: self.enabled,
               passes: passes,
               target: out,
           })
    }
}
