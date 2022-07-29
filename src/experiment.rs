// # Whether to raise when the control and candidate mismatch.
// # If this is nil, raise_on_mismatches class attribute is used instead.
// attr_accessor :raise_on_mismatches
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use crate::errors::{VictorsErrors, VictorsResult};
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
    pub cleaner: Option<fn(String)>,
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
            return Err(VictorsErrors::BehaviorNotUnique {
                experiment_name: (*&self.name).to_string(),
                name: name.to_string(),
            });
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
    pub fn clean<F>(&mut self, f: fn(String)) {
        self.cleaner = Some(f)
    }


    // fn raise_on_mismatch(&self) -> bool {
    //
    // }

    fn generate_result(&self, name: String) -> ExperimentResult<R> {
        let mut observations: Vec<Observation<R>> = vec![];

        // TODO: better way to get keys and shuffle?
        let mut keys = Vec::from_iter(self.behaviors.keys().cloned());
        keys.shuffle(&mut thread_rng());
        for key in keys {
            println!("found behavior...{}", key);
            let behavior = self.behaviors.get(&key);
            if let Some(behavior) = behavior {
                let behavior_results = behavior();
                observations.push(Observation::new(
                    key,
                    self.clone(),
                    behavior_results
                ));
            }

        }

        // TODO: get control
        // TODO: return result
        // control = observations.detect { |o| o.name == name }
        // Scientist::Result.new(self, observations, control)


        // TODO: change to ?
        let c = observations.iter().find(|o| o.name == name)
            .expect("should find name in observations");
        // TODO: can we change this to avoid the clones?
        return ExperimentResult::new(
            self.clone(),
            vec![],
            (*c).clone()
        );
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
    fn publish(&self) {}

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
            return Err(VictorsErrors::BehaviorMissing {
                experiment_name: (*&self.name).to_string(),
                name: name.to_string(),
            });
        }

        if let Some(before_block) = &self.before_run_block {
            before_block()
        }

        let result = self.generate_result(name.to_string());

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

// A null experiment.
// pub struct DefaultExperiment {
//     pub name: String,
//     pub behaviors: HashMap<String, fn()>,
//     pub run_if_block: Option<fn() -> bool>,
//     pub before_run_block: Option<fn()>,
//     pub cleaner: Option<fn(String)>,
// }
//
// impl DefaultExperiment {
//     fn run_if_block_allows(&self) -> bool {
//         match &self.run_if_block {
//             None => true,
//             Some(block) => block(),
//         }
//     }
//
//     // Register a named behavior for this experiment, default "candidate".
//     pub fn candidate(&mut self, name: &str, f: fn()) -> VictorsResult<()> {
//         self.add_behavior(name, f)
//     }
//
//     // Register the control behavior for this experiment.
//     pub fn control(&mut self, f: fn()) -> VictorsResult<()> {
//         self.add_behavior("control", f)
//     }
//
//     fn add_behavior(&mut self, name: &str, f: fn()) -> VictorsResult<()> {
//         if self.behaviors.contains_key(name) {
//             return Err(VictorsErrors::BehaviorNotUnique {
//                 experiment_name: (*&self.name).to_string(),
//                 name: name.to_string(),
//             });
//         }
//
//         self.behaviors.insert(name.to_string(), f);
//
//         return Ok(());
//     }
//
//     // Define a block of code to run before an experiment begins, if the experiment is enabled.
//     pub fn before_run(&mut self, f: fn()) {
//         self.before_run_block = Some(f)
//     }
//
//     // # A block to clean an observed value for publishing or storing.
//     // #
//     // # The block takes one argument, the observed value which will be cleaned.
//     // #
//     // # Returns the configured block.
//     // def clean(&block)
//     // @_scientist_cleaner = block
//     // end
//
//     // A block to clean an observed value for publishing or storing.
//     // The block takes one argument, the observed value which will be cleaned.
//     pub fn clean<F>(&mut self, f: fn(String)) {
//         self.cleaner = Some(f)
//     }
//
//
//     // fn raise_on_mismatch(&self) -> bool {
//     //
//     // }
//
//     fn generate_result(&self, name: String) {
//         let observations: Vec<Observation> = vec![];
//
//         // TODO: better way to get keys and shuffle?
//         let mut keys = Vec::from_iter(self.behaviors.keys().cloned());
//         keys.shuffle(&mut thread_rng());
//         for key in keys {
//             let behavior = self.behaviors.get(&key);
//         }
//
//     }
// }

// impl Experiment for DefaultExperiment {
//     // Don't run experiments.
//     fn is_enabled(&self) -> bool {
//         return false;
//     }
//
//     // Don't publish anything.
//     fn publish(&self) {}
//
//     fn run(&self, name: Option<&str>) -> VictorsResult<()> {
//         let name = if let Some(name) = name { name } else { "control" };
//
//         let block = self.behaviors.get(name);
//         if block.is_none() {
//             return Err(VictorsErrors::BehaviorMissing {
//                 experiment_name: (*&self.name).to_string(),
//                 name: "name".to_string(),
//             });
//         }
//
//         if let Some(before_block) = &self.before_run_block {
//             before_block()
//         }
//
//         // result = generate_result(name)
//
//         // begin
//         //     publish(result)
//         // rescue StandardError => ex
//         //     raised :publish, ex
//         // end
//
//         // if raise_on_mismatches? && result.mismatched?
//         //     if @_scientist_custom_mismatch_error
//         //         raise @_scientist_custom_mismatch_error.new(self.name, result)
//         //     else
//         //         raise MismatchError.new(self.name, result)
//         //     end
//         // end
//         //
//         // control = result.control
//         // raise control.exception if control.raised?
//         // control.value
//
//         return Ok(());
//     }
//
//     fn should_experiment_run(&self) -> bool {
//         return self.behaviors.len() > 1 && self.is_enabled() && self.run_if_block_allows();
//     }
//
// }

// # A mismatch, raised when raise_on_mismatches is enabled.
//   class MismatchError < Exception
//     attr_reader :name, :result

//     def initialize(name, result)
//       @name   = name
//       @result = result
//       super "experiment '#{name}' observations mismatched"
//     end

//     # The default formatting is nearly unreadable, so make it useful.
//     #
//     # The assumption here is that errors raised in a test environment are
//     # printed out as strings, rather than using #inspect.
//     def to_s
//       super + ":\n" +
//       format_observation(result.control) + "\n" +
//       result.candidates.map { |candidate| format_observation(candidate) }.join("\n") +
//       "\n"
//     end

//     def format_observation(observation)
//       observation.name + ":\n" +
//       if observation.raised?
//         lines = observation.exception.backtrace.map { |line| "    #{line}" }.join("\n")
//         "  #{observation.exception.inspect}" + "\n" + lines
//       else
//         "  #{observation.cleaned_value.inspect}"
//       end
//     end
//   end

//   module RaiseOnMismatch
//     # Set this flag to raise on experiment mismatches.
//     #
//     # This causes all science mismatches to raise a MismatchError. This is
//     # intended for test environments and should not be enabled in a production
//     # environment.
//     #
//     # bool - true/false - whether to raise when the control and candidate mismatch.
//     def raise_on_mismatches=(bool)
//       @raise_on_mismatches = bool
//     end

//     # Whether or not to raise a mismatch error when a mismatch occurs.
//     def raise_on_mismatches?
//       @raise_on_mismatches
//     end
//   end
