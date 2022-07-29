use crate::experiment::Experiment;

// Observation really only needs experiment to get cleaned value.
// instead of passing in experiment and calling into it to get clean_value
// experiment.clean_value has the following logic
//   # Clean a value with the configured clean block, or return the value
//   # if no clean block is configured.
//   #
//   # Rescues and reports exceptions in the clean block if they occur.

/// What happened when this named behavior was executed? Immutable.
// TODO: need to change value
#[derive(Clone)]
pub struct Observation<R: Clone>  {
    pub experiment: Experiment<R>,
    pub name: String,
    pub value: R, // what type should this be?
    pub exception: String, // probably dont need this
    pub duration: usize
}

impl<R: Clone> Observation<R> {

    // TODO: pass in lambda/function block which is executed and duration/value returned
    pub fn new(name: String, experiment: Experiment<R>, value: R) -> Self {
        return Self {
            name,
            value,
            exception: String::from(""), // TODO: change to Error
            experiment,
            duration: 0
        }
    }

    pub fn clean_value() {
        // TODO: Return experiment clean_value option
    }

    // TODO: equivalent_to fn
}

// impl<T> Copy for Foo<T> {}

// impl<R> Clone for Observation<R> {
//     fn clone(&self) -> Self {
//         *self
//     }
// }
