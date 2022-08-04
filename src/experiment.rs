use std::collections::HashMap;
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::value::{to_value, Map, Value};
use crate::context::Context;
use crate::errors::{BehaviorMissing, BehaviorNotUnique, MismatchError, VictorsErrors, VictorsResult};
use crate::experiment_result::ExperimentResult;
use crate::observation::Observation;
use crate::result_publisher::{NoopPublisher, Publisher};


static CONTROL_NAME: &str = "control";
static DEFAULT_CANDIDATE_NAME: &str = "candidate";
static DEFAULT_EXPERIMENT_NAME: &str = "experiment";

pub trait Experimentation
{
    type Result;

    // type T: Display = String;
    type Ignore: FnOnce(&Observation<Self::Result>, &Observation<Self::Result>) -> bool;

    type EnabledFn: Fn() -> bool;

    type BeforeRunFn: Fn();

    type RunIfFn: FnOnce() -> bool;

    type CleanerFn: Fn(Self::Result);

    type ComparatorFn: Fn(&Self::Result, &Self::Result) -> bool;

    type ErrorComparatorFn: Fn(&String, &String) -> bool;

    fn run(&self, name: Option<&str>) -> Self::Result;

    fn enabled(&mut self, enabled: Self::EnabledFn);

    fn is_enabled(&self) -> bool;

    fn before_run_block(&mut self, block: Self::BeforeRunFn);

    fn run_if(&mut self, block: Self::RunIfFn);

    fn add_ignore(&mut self, block: Self::Ignore);

    fn cleaner(&mut self, block: Self::CleanerFn);

    // fn publisher<T: Publisher<R> + 'a>(&mut self, publisher: T);

    fn comparator(&mut self, block: Self::ComparatorFn);

    fn error_comparator(&mut self, block: Self::ErrorComparatorFn);

    fn add_context(&mut self, context: Context);
}

// pub trait Experiment {
//     // TODO: allow sampling which can be done see ramping up
//     fn is_enabled(&self) -> bool;
//
//     fn publish(&self);
//
//     fn run(&self, name: Option<&str>) -> VictorsResult<()>;
//
//     fn should_experiment_run(&self) -> bool;
// }


// TODO: do we want to rename Experiment to ControlledExperiment
// Then make Experiment a trait (or Experimentation).

// TODO: make type alias for these various fn blocks
// TODO:
type BehaviorBlock<R> = fn() -> R;
// type BehaviorBlock<R> = dyn FnOnce() -> R;
type BoxedBehaviorBlock<R> = Box<BehaviorBlock<R>>;
type RunIfBlock = fn() -> bool;
type BeforeRunBlock = fn();
type CleanerBlock<R> = fn(R);
type EnabledFn = fn() -> bool;
// type IgnoresBlock<R> = Box<dyn FnOnce(&Observation<R>, &Observation<R>) -> bool>;
type IgnoresBlock<R> = fn(&Observation<R>, &Observation<R>) -> bool;
type ValueComparator<R> = fn(a: &R, b: &R) -> bool;
type ErrorComparator = fn(a: &String, b: &String) -> bool;
// type PublisherBlock<R> = Box<dyn Publisher<ExperimentResult<R>>>;
// type PublisherBlock<R> = fn(result: &ExperimentResult<R>);

// TODO: make context a struct?

pub struct Experiment<'a, R: Clone + PartialEq> {
    pub name: String,

    // TODO: probably need to have behaviors return results
    behaviors: HashMap<String, BehaviorBlock<R>>,

    // Sometimes you don't want an experiment to run. Say, disabling a new codepath for anyone
    // who isn't staff. You can disable an experiment by setting a run_if block.
    // If this returns false, the experiment will merely return the control value.
    // Otherwise, it defers to the experiment's configured enabled? method.

    /// Used to disable an experiment. If this returns false the experiment will merely return the
    /// control value. Otherwise it defers to the experiment's configured enabled method.
    pub run_if_block: Option<RunIfBlock>,
    pub before_run_block: Option<BeforeRunBlock>,
    pub cleaner: Option<CleanerBlock<R>>,
    pub enabled: EnabledFn,
    pub context: Context, // TODO: maybe AHashMap<String, Box<dyn Any>>, https://github.com/actix/actix-web/blob/7dc034f0fb70846d9bb3445a2414a142356892e1/actix-http/src/extensions.rs
    ignores: Vec<IgnoresBlock<R>>, //Vec<fn(&Observation<R>, &Observation<R>) -> bool>, // TODO: might need to return Result<bool>
    pub err_on_mismatches: bool,
    comparator: Option<ValueComparator<R>>,
    error_comparator: Option<ErrorComparator>,
    pub publisher: Box<dyn Publisher<R> + 'a>,
}

// impl<F: FnOnce(&Observation<R>, &Observation<R>) -> bool, R: Clone + PartialEq> Experimentation<F, R> for Experiment<R> {
//     fn add_ignore(&mut self, block: F) {
//         self.ignores.push(block);
//     }
// }

impl<'a, R: Clone + PartialEq> Experiment<'a, R> {

    /// Creates a new experiment with the name "experiment"
    pub fn default() -> Self {
        return Experiment::new(DEFAULT_EXPERIMENT_NAME);
    }

    /// Creates a new experiment
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    pub fn new(name: &'static str) -> Self {
        return Self {
            name: name.to_string(),
            behaviors: Default::default(),
            run_if_block: None,
            before_run_block: None,
            cleaner: None,
            enabled: || { true },
            context: Default::default(),
            ignores: vec![],
            err_on_mismatches: false,
            comparator: None,
            error_comparator: None,
            // publisher: |result| {}
            publisher: Box::new(NoopPublisher{}),
        }
    }

    /// Creates a new experiment with initial context
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `context` - Map of extra experiment data
    pub fn new_with_context(name: &'static str, context: Context) -> Self {
        return Self {
            name: name.to_string(),
            behaviors: Default::default(),
            run_if_block: None,
            before_run_block: None,
            cleaner: None,
            enabled: || { true },
            context,
            ignores: vec![],
            err_on_mismatches: false,
            comparator: None,
            error_comparator: None,
            // publisher: |result| {}
            publisher: Box::new(NoopPublisher{})
        }
    }

    /// Define a block that determines whether or not the candidate experiments should run.
    pub fn run_if(&mut self, block: RunIfBlock) {
        self.run_if_block = Some(block);
    }

    fn run_if_block_allows(&self) -> bool {
        match &self.run_if_block {
            None => true,
            Some(block) => block(),
        }
    }

    /// Register a named behavior for this experiment, default "candidate".
    pub fn candidate(&mut self, name: &str, f: BehaviorBlock<R>) -> VictorsResult<()> {
        self.add_behavior(name, f)
    }

    /// Register the control behavior for this experiment.
    pub fn control(&mut self, f: BehaviorBlock<R>) -> VictorsResult<()> {
        self.add_behavior(CONTROL_NAME, f)
    }

    fn add_behavior(&mut self, name: &str, f: BehaviorBlock<R>) -> VictorsResult<()> {
        if self.behaviors.contains_key(name) {
            return Err(VictorsErrors::BehaviorNotUnique(BehaviorNotUnique {
                experiment_name: (*&self.name).to_string(),
                name: name.to_string(),
            }));
        }
        self.behaviors.insert(name.to_string(), f);

        return Ok(());
    }

    /// Define a block of code to run before an experiment begins, if the experiment is enabled.
    pub fn before_run(&mut self, f: BeforeRunBlock) {
        self.before_run_block = Some(f)
    }

    /// A block to clean an observed value for publishing or storing.
    /// The block takes one argument, the observed value which will be cleaned.
    pub fn clean(&mut self, f: CleanerBlock<R>) {
        self.cleaner = Some(f)
    }


    fn generate_result(&self, name: String) -> VictorsResult<ExperimentResult<R>> {
        let mut observations = vec![];
        let mut observation_to_return_index = None;

        // TODO: better way to get keys and shuffle?
        let mut keys = Vec::from_iter(self.behaviors.keys().cloned());
        keys.shuffle(&mut thread_rng());
        for (i, key) in keys.iter().enumerate() {
            let behavior = self.behaviors.get(key);
            if let Some(behavior) = behavior {
                let start = Instant::now();
                let behavior_results = behavior();
                let duration = start.elapsed();
                // TODO: need to clean value at some point
                let observation = Observation::new(
                    key.to_string(),
                    self.name.to_string(),
                    behavior_results,
                    None,
                    duration.as_millis()
                );

                observations.push(observation);
                if key == &name {
                    observation_to_return_index = Some(i);
                }
            }
        }

        match observation_to_return_index {
            None => {
                Err(VictorsErrors::BehaviorMissing(BehaviorMissing {
                    experiment_name: self.name.to_string(),
                    name
                }))
            }
            Some(o) => {
                Ok(ExperimentResult::new(
                    &self,
                    observations,
                    o
                ))
            }
        }
    }

    pub fn add_context(&mut self, context: Context) {
        self.context.extend(context);
    }

    /// Configure experiment to ignore observations based on the given block.
    ///
    /// # Arguments
    /// * `ignore_block` - Function takes two arguments, the control observation an the candidate
    ///                    observation which didn't match the control. If the block returns true the
    ///                    mismatch is disregarded.
    pub fn add_ignore(&mut self, ignore_block: IgnoresBlock<R>) {
        self.ignores.push(ignore_block)
    }

    /// Ignore a mismatched observation
    ///
    /// Iterates through the configured ignore blocks and calls each of them with the given
    /// control and mismatched candidate observations.
    ///
    /// # Arguments
    /// * `control` - the control observation
    /// * `candidate` - the candidate observation
    ///
    /// # Return
    /// whether or not to ignore mismatch observation
    pub fn ignore_mismatch_observation(
        &self,
        control: &Observation<R>,
        candidate: &Observation<R>
    ) -> bool {
        if self.ignores.is_empty() {
            return false;
        }

        // for n in 1..self.ignores.len() {
        //     if self.ignores[n](&control, &candidate) {
        //         return true;
        //     }
        // }

        return self.ignores.iter().any(|ignore| ignore(&control, &candidate));
        // for ignore in &self.ignores {
        //     if ignore(&control, &candidate) {
        //         return true;
        //     }
        // }
        // return false;
    }

    // TODO: does this need to return a result?
    pub fn observations_are_equivalent(&self, a: &Observation<R>, b: &Observation<R>) -> bool {
        return a.equivalent_to(b, self.comparator, self.error_comparator);
    }

    fn enabled(&mut self, enabled: fn() -> bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        return (self.enabled)();
    }

    // // Don't publish anything.
    // fn publish(&self, result: &ExperimentResult<R>) {}
    pub fn result_publisher<T: Publisher<R> + 'a>(&mut self, publisher: T) {
        self.publisher = Box::new(publisher);
    }

    // TODO: run needs to return a generic result

    // TODO: not sure I love passing in name to mean return result from behavior with name
    // I think its primarily used when there is no control and just candidate
    // see if there is a better way to do that or just have a separate fn perhaps
    // Controlled vs uncontrolled experiment.
    // These could be traits or separate structs. uncontrolled would have run with name
    // would a enum help here?
    // issue is that they share similar behavior

    // Need a fn to run control/candidates in parallel

    /// Run all the behaviors for this experiment, observing each and publishing the results.
    /// Return the result of the control
    /// See [internal_run]
    pub fn run(&mut self) -> VictorsResult<R> {
        return self.internal_run(CONTROL_NAME);
    }

    /// Run all the behaviors for this experiment, observing each and publishing the results.
    /// Return the result of the named behavior
    ///
    /// # Arguments
    /// * `name`
    pub(crate) fn internal_run(&mut self, name: &str) -> VictorsResult<R> {
        let block = self.behaviors.get(name);
        match block {
            None => {
                return Err(VictorsErrors::BehaviorMissing(BehaviorMissing {
                    experiment_name: (*&self.name).to_string(),
                    name: name.to_string(),
                }));
            }
            Some(block) => {
                if !self.should_experiment_run() {
                    return Ok(block());
                }
            }
        }

        if let Some(before_block) = &self.before_run_block {
            before_block()
        }

        let result = self.generate_result(name.to_string())?;
        // TODO: this should return a VictorsError<()> to handle errors?
        // ruby version has a `raised` fn that takes in operation and error and allows users to
        // customize behavior. Default behavior is to re-raise the exception
        self.publisher.publish(&result);

        if self.err_on_mismatches && result.matched() {
            // TODO: do we want to support custom mismatch error?
            // ruby version has a `raise_with` fn that sets `@_scientist_custom_mismatch_error`
            // to allow for a custom err to be raised
            return Err(VictorsErrors::MismatchError(MismatchError {
                experiment_name: self.name.to_string(),
                exception: None,
                message: "".to_string(),
                backtrace: None
            }));
        }

        // if let Some(err) = &result.control.exception {
        //     todo!()
        // } else {
        //     return Ok(result.control.value.to_owned());
        // }

        // TODO: fix unwrap
        return Ok(result.control().unwrap().value.to_owned());
    }


    fn should_experiment_run(&self) -> bool {
        return self.behaviors.len() > 1 && self.is_enabled() && self.run_if_block_allows();
    }

    /// Whether to return an error when the control and candidate mismatch.
    fn err_on_mismatches(&mut self, err_on_mismatches: bool) {
        self.err_on_mismatches = err_on_mismatches;
    }

    /// A block which compares two experimental values.
    ///
    /// # Arguments
    /// * `comparator` - The block must take two arguments, the control value and a candidate value,
    ///                  and return true or false.
    pub fn comparator(&mut self, comparator: fn(a: &R, b: &R) -> bool) {
        self.comparator = Some(comparator);
    }

    /// A block which compares two experimental errors.
    ///
    /// # Arguments
    /// * `comparator` - The block must take two arguments, the control error and a candidate error,
    ///                  and return true or false.
    pub fn error_comparator(&mut self, comparator: ErrorComparator) {
        self.error_comparator = Some(comparator);
    }

}

pub struct UncontrolledExperiment<'a, R: Clone + PartialEq> {
    experiment: Experiment<'a, R>
}

impl<'a, R: Clone + PartialEq> UncontrolledExperiment<'a, R> {

    /// Creates a new experiment with the name "experiment"
    pub fn default() -> Self {
        return Self {
            experiment: Experiment::default()
        }
    }

    /// Creates a new experiment
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    pub fn new(name: &'static str) -> Self {
        return Self {
            experiment: Experiment::new(name)
        }
    }

    /// Creates a new experiment with initial context
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `context` - Map of extra experiment data
    pub fn new_with_context(name: &'static str, context: Context) -> Self {
        return Self {
            experiment: Experiment::new_with_context(name, context)
        }
    }

    fn run_if_block_allows(&self) -> bool {
        self.experiment.run_if_block_allows()
    }

    /// Register a named behavior for this experiment, default "candidate".
    pub fn candidate(&mut self, name: &str, f: BehaviorBlock<R>) -> VictorsResult<()> {
        self.experiment.candidate(name, f)
    }

    /// Define a block of code to run before an experiment begins, if the experiment is enabled.
    pub fn before_run(&mut self, f: BeforeRunBlock) {
        self.experiment.before_run(f)
    }

    /// A block to clean an observed value for publishing or storing.
    /// The block takes one argument, the observed value which will be cleaned.
    pub fn clean(&mut self, f: CleanerBlock<R>) {
        self.experiment.clean(f)
    }

    pub fn add_context(&mut self, context: Context) {
        self.experiment.add_context(context)
    }

    /// Configure experiment to ignore observations based on the given block.
    ///
    /// # Arguments
    /// * `ignore_block` - Function takes two arguments, the control observation an the candidate
    ///                    observation which didn't match the control. If the block returns true the
    ///                    mismatch is disregarded.
    pub fn add_ignore(&mut self, ignore_block: IgnoresBlock<R>) {
        self.experiment.add_ignore(ignore_block)
    }

    /// See [Experiment::ignore_mismatch_observation]
    pub fn ignore_mismatch_observation(
        &self,
        control: &Observation<R>,
        candidate: &Observation<R>
    ) -> bool {
        self.experiment.ignore_mismatch_observation(control, candidate)
    }

    fn enabled(&mut self, enabled: fn() -> bool) {
        self.experiment.enabled(enabled)
    }

    fn is_enabled(&self) -> bool {
        return (self.experiment.enabled)();
    }

    // fn publish(&self, result: &ExperimentResult<R>) {
    //     self.experiment.publish(result)
    // }


    /// Run all the behaviors for this experiment, observing each and publishing the results.
    /// Return the result of the named candidate
    /// See [Experiment::internal_run]
    pub fn run(&mut self, name: &'static str) -> VictorsResult<R> {
        return self.experiment.internal_run(name);
    }

    fn should_experiment_run(&self) -> bool {
        return self.experiment.should_experiment_run();
    }

    pub fn observations_are_equivalent(&self, a: &Observation<R>, b: &Observation<R>) -> bool {
        return self.experiment.observations_are_equivalent(a, b);
    }

    /// Whether to return an error when the control and candidate mismatch.
    fn err_on_mismatches(&mut self, err_on_mismatches: bool) {
        self.experiment.err_on_mismatches(err_on_mismatches);
    }
}
