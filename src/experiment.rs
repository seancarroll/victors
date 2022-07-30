// # Whether to raise when the control and candidate mismatch.
// # If this is nil, raise_on_mismatches class attribute is used instead.
// attr_accessor :raise_on_mismatches
use std::collections::HashMap;
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use crate::errors::{BehaviorMissing, BehaviorNotUnique, VictorsErrors, VictorsResult};
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



#[derive(Clone)]
pub struct Experiment<R: Clone> {
    pub name: String,
    // TODO: separate out control and then have map of candidates
    behaviors: HashMap<String, fn() -> R>,
    pub run_if_block: Option<fn() -> bool>,
    pub before_run_block: Option<fn()>,
    pub cleaner: Option<fn(R)>,
    pub enabled: fn() -> bool,
    context: HashMap<String, Value>
}

impl<R: Clone> Experiment<R> {

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
            context: Default::default()
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
            context
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
        let mut candidate_observations: Vec<Observation<R>> = vec![];
        let mut observation_to_return = None;

        // TODO: better way to get keys and shuffle?
        let mut keys = Vec::from_iter(self.behaviors.keys().cloned());
        keys.shuffle(&mut thread_rng());
        for key in keys {
            let behavior = self.behaviors.get(&key);
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
                if key == name {
                    observation_to_return = Some(observation);
                } else {
                    candidate_observations.push(observation);
                }
            }
        }

        match observation_to_return {
            None => {
                Err(VictorsErrors::BehaviorMissing(BehaviorMissing {
                    experiment_name: self.name.to_string(),
                    name
                }))
            }
            Some(o) => {
                Ok(ExperimentResult::new(
                    self,
                    candidate_observations,
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
        self.publish(&result);
        // begin
        //     publish(result)
        // rescue StandardError => ex
        //     raised :publish, ex
        // end

        // if raise_on_mismatches? && result.mismatched?
        //     if @_scientist_custom_mismatch_error
        //         raise @_scientist_custom_mismatch_error.new(self.name, result)
        //     else
        //         raise MismatchError.new(self.name, result)
        //     end
        // end
        //
        // control = result.control
        // raise control.exception if control.raised?
        // control.value

        return Ok(result.control.value);
    }

    fn should_experiment_run(&self) -> bool {
        return self.behaviors.len() > 1 && self.is_enabled() && self.run_if_block_allows();
    }
}

pub struct UncontrolledExperiment<R: Clone> {
    experiment: Experiment<R>
}

impl<R: Clone> UncontrolledExperiment<R> {

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
}
