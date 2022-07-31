// # Whether to raise when the control and candidate mismatch.
// # If this is nil, raise_on_mismatches class attribute is used instead.
// attr_accessor :raise_on_mismatches
use std::collections::HashMap;
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use crate::errors::{BehaviorMissing, BehaviorNotUnique, MismatchError, VictorsErrors, VictorsResult};
use crate::experiment_result::ExperimentResult;
use crate::observation::Observation;


static CONTROL_NAME: &str = "control";
static DEFAULT_CANDIDATE_NAME: &str = "candidate";
static DEFAULT_EXPERIMENT_NAME: &str = "experiment";

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

#[derive(Clone)]
pub struct Experiment<R: Clone + PartialEq> {
    pub name: String,
    // TODO: separate out control and then have map of candidates
    behaviors: HashMap<String, fn() -> R>,
    pub run_if_block: Option<fn() -> bool>,
    pub before_run_block: Option<fn()>,
    pub cleaner: Option<fn(R)>,
    pub enabled: fn() -> bool,
    context: HashMap<String, Value>,
    ignores: Vec<fn(&Observation<R>, &Observation<R>) -> bool>, // TODO: might need to return Result<bool>
    pub err_on_mismatches: bool,
    comparator: Option<fn(a: &R, b: &R) -> bool>,
    error_comparator: Option<fn(a: &VictorsErrors, b: &VictorsErrors) -> bool>
}

impl<R: Clone + PartialEq> Experiment<R> {

    /// Creates a new experiment with the name "experiment"
    pub fn default() -> Self {
        return Experiment::new("experiment");
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
            error_comparator: None
        }
    }

    /// Creates a new experiment with initial context
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `context` - Map of extra experiment data
    pub fn new_with_context(name: &'static str, context: HashMap<String, Value>) -> Self {
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
            error_comparator: None
        }
    }

    fn run_if_block_allows(&self) -> bool {
        match &self.run_if_block {
            None => true,
            Some(block) => block(),
        }
    }

    // Register a named behavior for this experiment, default "candidate".
    pub fn candidate(&mut self, name: &str, f: fn() -> R) -> VictorsResult<()> {
        self.add_behavior(name, f)
    }

    // Register the control behavior for this experiment.
    pub fn control(&mut self, f: fn() -> R) -> VictorsResult<()> {
        self.add_behavior("control", f)
    }

    fn add_behavior(&mut self, name: &str, f: fn() -> R) -> VictorsResult<()> {
        if self.behaviors.contains_key(name) {
            return Err(VictorsErrors::BehaviorNotUnique(BehaviorNotUnique {
                experiment_name: (*&self.name).to_string(),
                name: name.to_string(),
            }));
        }
        self.behaviors.insert(name.to_string(), f);

        return Ok(());
    }

    // Define a block of code to run before an experiment begins, if the experiment is enabled.
    pub fn before_run(&mut self, f: fn()) {
        self.before_run_block = Some(f)
    }

    // # A block to clean an observed value for publishing or storing.
    // #
    // # The block takes one argument, the observed value which will be cleaned.
    // #
    // # Returns the configured block.
    // def clean(&block)
    // @_scientist_cleaner = block
    // end

    // A block to clean an observed value for publishing or storing.
    // The block takes one argument, the observed value which will be cleaned.
    pub fn clean(&mut self, f: fn(R)) {
        self.cleaner = Some(f)
    }


    // fn raise_on_mismatch(&self) -> bool {
    //
    // }

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

        // TODO: get control
        // TODO: return result
        // control = observations.detect { |o| o.name == name }
        // Scientist::Result.new(self, observations, control)


        // TODO: change to ?
        // let c = observations.into_iter().find(|o| o.name == name)
        //     .expect("should find name in observations");
        // TODO: can we change this to avoid the clones?
        // return Ok(ExperimentResult::new(
        //     self,
        //     candidate_observations,
        //     observation_to_return.
        // ));
    }

    pub fn add_context(&mut self, context: HashMap<String, Value>) {
        self.context.extend(context);
    }

    /// Configure experiment to ignore observations based on the given block.
    ///
    /// # Arguments
    /// * `ignore_block` - Function takes two arguments, the control observation an the candidate
    ///                    observation which didn't match the control. If the block returns true the
    ///                    mismatch is disregarded.
    pub fn add_ignore(&mut self, ignore_block: fn(&Observation<R>, &Observation<R>) -> bool) {
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

        return self.ignores.iter().any(|ignore| ignore(&control, &candidate));
    }

    // TODO: does this need to return a result?
    pub fn observations_are_equivalent(&self, a: &Observation<R>, b: &Observation<R>) -> bool {
        // a.equivalent_to(b, self.comparator, self.error_comparator)
        return false;
    }

    fn enabled(&mut self, enabled: fn() -> bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        return (self.enabled)();
    }

    // Don't publish anything.
    fn publish(&self, result: &ExperimentResult<R>) {}

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
    pub fn run(&self) -> VictorsResult<R> {
        return self.internal_run(CONTROL_NAME);
    }

    /// Run all the behaviors for this experiment, observing each and publishing the results.
    /// Return the result of the named behavior
    pub(crate) fn internal_run(&self, name: &str) -> VictorsResult<R> {
        let block = self.behaviors.get(name);
        if block.is_none() {
            return Err(VictorsErrors::BehaviorMissing(BehaviorMissing {
                experiment_name: (*&self.name).to_string(),
                name: name.to_string(),
            }));
        }

        if let Some(before_block) = &self.before_run_block {
            before_block()
        }

        let result = self.generate_result(name.to_string())?;
        // TODO: this should return a VictorsError<()> to handle
        // ruby version has a `raised` fn that takes in operation and error and allows users to
        // customize behavior. Default behavior is to re-raise the exception
        self.publish(&result);

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
    pub fn comparator(&mut self, comparator: fn(&R, b: &R) -> bool) {
        self.comparator = Some(comparator);
    }

    /// A block which compares two experimental errors.
    ///
    /// # Arguments
    /// * `comparator` - The block must take two arguments, the control error and a candidate error,
    ///                  and return true or false.
    pub fn error_comparator(&mut self, comparator: fn(&VictorsErrors, b: &VictorsErrors) -> bool) {
        self.error_comparator = Some(comparator);
    }

}

pub struct UncontrolledExperiment<R: Clone + PartialEq> {
    experiment: Experiment<R>
}

impl<R: Clone + PartialEq> UncontrolledExperiment<R> {

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
    pub fn new_with_context(name: &'static str, context: HashMap<String, Value>) -> Self {
        return Self {
            experiment: Experiment::new_with_context(name, context)
        }
    }

    fn run_if_block_allows(&self) -> bool {
        self.experiment.run_if_block_allows()
    }

    // Register a named behavior for this experiment, default "candidate".
    pub fn candidate(&mut self, name: &str, f: fn() -> R) -> VictorsResult<()> {
        self.experiment.candidate(name, f)
    }

    // Define a block of code to run before an experiment begins, if the experiment is enabled.
    pub fn before_run(&mut self, f: fn()) {
        self.experiment.before_run(f)
    }

    // A block to clean an observed value for publishing or storing.
    // The block takes one argument, the observed value which will be cleaned.
    pub fn clean(&mut self, f: fn(R)) {
        self.experiment.clean(f)
    }

    pub fn add_context(&mut self, context: HashMap<String, Value>) {
        self.experiment.add_context(context)
    }

    /// Configure experiment to ignore observations based on the given block.
    ///
    /// # Arguments
    /// * `ignore_block` - Function takes two arguments, the control observation an the candidate
    ///                    observation which didn't match the control. If the block returns true the
    ///                    mismatch is disregarded.
    pub fn add_ignore(&mut self, ignore_block: fn(&Observation<R>, &Observation<R>) -> bool) {
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

    fn publish(&self, result: &ExperimentResult<R>) {
        self.experiment.publish(result)
    }


    /// Run all the behaviors for this experiment, observing each and publishing the results.
    /// Return the result of the named candidate
    /// See [Experiment::internal_run]
    pub fn run(&self, name: &'static str) -> VictorsResult<R> {
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
