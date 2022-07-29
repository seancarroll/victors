use crate::experiment::Experiment;

// Observation really only needs experiment to get cleaned value.
// instead of passing in experiment and calling into it to get clean_value
// experiment.clean_value has the following logic
//   # Clean a value with the configured clean block, or return the value
//   # if no clean block is configured.
//   #
//   # Rescues and reports exceptions in the clean block if they occur.

/// What happened when this named behavior was executed? Immutable.
#[derive(Clone)]
pub struct Observation<R: Clone>  {
    /// The experiment this observation is for
    pub experiment_name: String,
    /// name of the behavior
    pub name: String,
    pub value: R,
    /// cleaned value suitable for publishing. See [Experiment::cleaner] block. None if no cleaner
    pub cleaned_value: Option<R>, // what type should this be?
    pub exception: Option<String>, // TODO: change to error
    pub duration: u128
}

impl<R: Clone> Observation<R> {

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
            exception: None,
            experiment_name,
            duration
        }
    }

    pub fn clean_value() {
        // TODO: Return experiment clean_value option
    }

    // TODO: equivalent_to fn
}
