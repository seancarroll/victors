use crate::{context::Context, experiment::Experiment, observation::Observation};

/// The immutable result of running an experiment.
#[derive(Clone, PartialEq)]
pub struct ExperimentResult<R: Clone + PartialEq> {
    experiment_name: String,
    observations: Vec<Observation<R>>,
    context: Context,
    // TODO: I dont love using "control" here as its abusing the name for uncontrolled experiments
    // need to find a better term. Maybe primary behavior
    control_index: usize,
    mismatched_indexes: Vec<usize>,
    ignored_indexes: Vec<usize>,
}

impl<'a, R: Clone + PartialEq> ExperimentResult<R> {

    /// Create a new experiment result
    ///
    /// # Arguments
    /// * `experiment`
    /// * `observations`
    /// * `control_index`
    pub fn new(
        experiment: &'a Experiment<'_, R>,
        observations: Vec<Observation<R>>,
        control_index: usize
    ) -> Self {
        let (mismatched_indexes, ignored_indexes) =
            ExperimentResult::evaluate_candidates(experiment, &observations, control_index);
        Self {
            experiment_name: experiment.name.to_string(),
            observations,
            context: experiment.context.clone(),
            control_index,
            mismatched_indexes,
            ignored_indexes,
        }
    }

    /// Returns experiment name corresponding to the results
    pub fn experiment_name(&self) -> &String {
        return &self.experiment_name;
    }

    // TODO: rename as i dont like "control"
    /// Returns the control observation
    pub fn control(&self) -> Option<&Observation<R>> {
        return self.observations.get(self.control_index);
    }

    /// Returns reference to the experiment context
    pub fn context(&self) -> &Context {
        return &self.context;
    }

    /// Return mismatched observations
    pub fn mismatched(&self) -> Vec<&Observation<R>> {
        let mut mismatched = vec![];
        for i in &self.mismatched_indexes {
            if let Some(observation) = self.observations.get(*i) {
                mismatched.push(observation);
            }
        }
        return mismatched;
    }

    /// return ignored observations
    pub fn ignored(&self) -> Vec<&Observation<R>> {
        let mut ignores = vec![];
        for i in &self.ignored_indexes {
            if let Some(observation) = self.observations.get(*i) {
                ignores.push(observation);
            }
        }
        return ignores;
    }

    /// Returns if the result a match between all behaviors?
    pub fn matched(&self) -> bool {
        return self.mismatched_indexes.is_empty() && !self.has_ignores();
    }

    /// Returns if there were mismatches in the behaviors?
    pub fn has_mismatches(&self) -> bool {
        return !self.mismatched_indexes.is_empty();
    }

    pub fn has_ignores(&self) -> bool {
        return !self.ignored_indexes.is_empty();
    }

    // TODO: can evaluate candidate outside and then dont have to worry about lifetime
    /// Evaluate the candidates to find mismatched and ignored results.
    fn evaluate_candidates(
        experiment: &'a Experiment<'_, R>,
        observations: &'a Vec<Observation<R>>,
        control_index: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut mismatched = vec![];
        let mut ignored = vec![];
        // TODO: what to do here. we enforce this so it shouldnt happen.
        let control = observations.get(control_index).unwrap();
        for (i, observation) in observations.iter().enumerate() {
            let is_equivalent = experiment.observations_are_equivalent(&control, observation);
            if !is_equivalent {
                let ignore = experiment.ignore_mismatch_observation(&control, observation);
                if ignore {
                    ignored.push(i);
                } else {
                    mismatched.push(i);
                }
            }
        }

        return (mismatched, ignored);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Experiment, ExperimentResult, Observation};

    #[test]
    fn should_evaluate_observations() {
        let mut experiment = Experiment::default();
        let a = create_observation("a", 1);
        let b = create_observation("b", 1);

        let result = ExperimentResult::new(
            &experiment,
            vec![a, b],
            1
        );

        assert!(result.matched());
        assert!(!result.has_mismatches());
        assert!(result.mismatched().is_empty());
    }

    #[test]
    fn should_have_no_mismatches_if_there_is_only_a_control_observation() {

    }

    #[test]
    fn should_evaluate_observations_using_experiments_compare_block() {

    }

    #[test]
    fn should_not_ignore_any_mismatches_when_nothings_ignored() {

    }

    #[test]
    fn should_use_experiments_ignore_block_to_ignore_mismatched_observations() {

    }

    #[test]
    fn should_partition_observations_into_mismatched_and_ignored_when_applicable() {

    }

    #[test]
    fn should_expose_experiment_name() {

    }

    #[test]
    fn should_expose_context_from_experiment() {

    }

    fn create_observation(name: &'static str, value: u8) -> Observation<u8> {
        return Observation::new(
            name.to_string(),
            "experiment".to_string(),
            value,
            None,
            1
        );
    }
}
