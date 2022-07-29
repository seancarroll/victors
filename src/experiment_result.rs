use crate::experiment::Experiment;
use crate::observation::Observation;

/// The immutable result of running an experiment.
pub struct ExperimentResult<'a, R: Clone> {
    pub candidates: Vec<Observation<R>>,
    pub control: Observation<R>,
    // pub experiment: &'a dyn Experiment,
    pub experiment: &'a Experiment<R>,
    pub ignored: Vec<Observation<R>>,
    pub mismatched: Vec<Observation<R>>,
    pub observations: Vec<Observation<R>>,
}

impl<'a, R: Clone> ExperimentResult<'a, R> {
    pub fn new(experiment: &'a Experiment<R>, observations: Vec<Observation<R>>, control: Observation<R>) -> Self {
        // evaluate_candidates();
        let result = Self {
            candidates: vec![],
            experiment,
            observations,
            control,
            mismatched: vec![],
            ignored: vec![],
        };

        return result;
    }

    pub fn matched(&self) -> bool {
        return self.mismatched.is_empty() && !self.has_ignores();
    }

    pub fn has_matched(&self) -> bool {
        return !self.mismatched.is_empty();
    }

    pub fn has_ignores(&self) -> bool {
        return !self.ignored.is_empty();
    }

    // Internal: evaluate the candidates to find mismatched and ignored results
    //
    // Sets @ignored and @mismatched with the ignored and mismatched candidates.
    fn evaluate_candidates() {
        // mismatched = candidates.reject do |candidate|
        //     experiment.observations_are_equivalent?(control, candidate)
        // end

        // @ignored = mismatched.select do |candidate|
        //     experiment.ignore_mismatched_observation? control, candidate
        // end

        // @mismatched = mismatched - @ignored
    }
}
