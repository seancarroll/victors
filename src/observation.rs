use std::time::Instant;
use crate::errors::VictorsErrors;
use serde::{Deserialize, Serialize};

// Observation really only needs experiment to get cleaned value.
// instead of passing in experiment and calling into it to get clean_value
// experiment.clean_value has the following logic
//   # Clean a value with the configured clean block, or return the value
//   # if no clean block is configured.
//   #
//   # Rescues and reports exceptions in the clean block if they occur.

// TODO: should R also include Copy?
/// What happened when this named behavior was executed? Immutable.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Observation<R: Clone + PartialEq + Serialize> {
    /// The experiment this observation is for
    pub experiment_name: String,
    /// name of the behavior
    pub name: String,
    pub value: R, // TODO: Does this need to be Option<R>
    /// cleaned value suitable for publishing. See [Experiment::cleaner] block. None if no cleaner
    pub cleaned_value: Option<R>, // TODO: what type should this be?
    // pub exception: Option<VictorsErrors>,
    pub duration: u128,
    // exception - should this be a Victor error or would it just be Err? How to attach backtrace?
}

impl<R: Clone + PartialEq + Serialize> Observation<R> {
    // TODO: pass in lambda/function block which is executed and duration/value returned
    pub fn new(
        name: String,
        experiment_name: String,
        value: R,
        cleaned_value: Option<R>,
        duration: u128
    ) -> Self {
        return Self {
            name,
            value,
            cleaned_value,
            // exception: None,
            experiment_name,
            duration,
        };
    }

    // TODO: should we have a Behavior struct with an execute method that returns an Observation?
    // or just have a method in experiment to execute behavior
    // TODO: how to handle clean value
    pub fn initialize<F>(
        name: String,
        experiment_name: String,
        f: F
    ) -> Self
    where
        F: Fn() -> R
    {
        let start = Instant::now();
        let value = f();
        let duration = start.elapsed();

        Self {
            name,
            experiment_name,
            value,
            duration: duration.as_millis(),
            cleaned_value: None
        }
    }

    pub fn clean_value() {
        // TODO: Return experiment clean_value option
    }

    // TODO: equivalent_to
    // not sure this needs to be a fn here
    /// Is this observation equivalent to another?
    pub fn equivalent_to(
        &self,
        other: &Observation<R>,
        comparator: Option<fn(a: &R, b: &R) -> bool>,
        error_comparator: Option<fn(a: &String, b: &String) -> bool>,
    ) -> bool {
        // TODO: check raise // error
        // if let (Some(exception),  Some(other_exception)) = (&self.exception, &other.exception) {
        //     return if let Some(error_comparator) = error_comparator {
        //         error_comparator(exception, other_exception)
        //     } else {
        //         exception == other_exception
        //     }
        // }

        return if let Some(comparator) = comparator {
            comparator(&self.value, &other.value)
        } else {
            self.value == other.value
        };
    }
}
