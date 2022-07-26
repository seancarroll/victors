use std::any::Any;
use crate::{context::Context, experiment::Experiment, observation::Observation};
use serde::{Deserialize, Serialize};

trait ExperimentValue: Clone {}

/// The immutable result of running an experiment.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExperimentResult<R: Clone + PartialEq + Serialize> {
    experiment_name: String,
    observations: Vec<Observation<R>>,
    // observations: Vec<Observation<dyn ExperimentValue + PartialEq>>,
    context: Context,
    // TODO: I dont love using "control" here as its abusing the name for uncontrolled experiments
    // need to find a better term. Maybe primary behavior
    control_index: usize,
    mismatched_indexes: Vec<usize>,
    ignored_indexes: Vec<usize>,
}

impl<'a, R: Clone + PartialEq + Serialize> ExperimentResult<R> {
// impl<'a> ExperimentResult {

    /// Create a new experiment result
    ///
    /// # Arguments
    /// * `experiment`
    /// * `observations`
    /// * `control_index`
    pub fn new(
    // pub fn new<R: ExperimentValue + PartialEq>(
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
    use std::cell::RefCell;
    use serde::Serialize;
    use serde_json::json;
    use crate::{Context, Experiment, ExperimentResult, Observation};

    // TODO: split this test?
    #[test]
    fn should_evaluate_observations() {
        let experiment = Experiment::default();
        let a = create_observation("a", 1);
        let b = create_observation("b", 1);

        let result = ExperimentResult::new(
            &experiment,
            vec![a, b],
            0
        );

        assert!(result.matched());
        assert!(!result.has_mismatches());
        assert!(result.mismatched().is_empty());

        let x = create_observation("x", 1);
        let y = create_observation("y", 2);
        let z = create_observation("z", 3);

        let result = ExperimentResult::new(
            &experiment,
            vec![x.clone(), y.clone(), z.clone()],
            0
        );

        assert!(!result.matched());
        assert!(result.has_mismatches());
        assert_eq!(vec![&y, &z], result.mismatched());
    }

    #[test]
    fn should_have_no_mismatches_if_there_is_only_a_control_observation() {
        let experiment = Experiment::default();
        let a = create_observation("a", 1);

        let result = ExperimentResult::new(
            &experiment,
            vec![a],
            0
        );

        assert!(result.matched());
    }

    #[test]
    fn should_evaluate_observations_using_experiments_compare_block() {
        let mut experiment = Experiment::default();
        experiment.comparator(|x: &&str, y: &&str| { x.eq_ignore_ascii_case(y) });
        let a = create_observation("a", "x");
        let b = create_observation("b", "X");


        let result = ExperimentResult::new(
            &experiment,
            vec![a, b],
            0
        );

        assert!(result.matched(), "{:?}", result.mismatched());
    }

    #[test]
    fn should_not_ignore_any_mismatches_when_nothings_ignored() {
        let experiment = Experiment::default();
        let a = create_observation("a", 1);
        let b = create_observation("b", 2);

        let result = ExperimentResult::new(
            &experiment,
            vec![a, b],
            0
        );

        assert!(result.has_mismatches());
        assert!(!result.has_ignores());
    }

    #[test]
    fn should_use_experiments_ignore_block_to_ignore_mismatched_observations() {
        let called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.add_ignore(|_a, _b| { called.replace(true); true });

        let a = create_observation("a", 1);
        let b = create_observation("b", 2);

        let result = ExperimentResult::new(
            &experiment,
            vec![a.clone(), b.clone()],
            0
        );

        assert!(!result.has_mismatches());
        assert!(result.mismatched().is_empty());
        assert!(!result.matched());
        assert!(result.has_ignores());
        assert_eq!(vec![&b], result.ignored());
        assert!(called.take());
    }

    #[test]
    fn should_partition_observations_into_mismatched_and_ignored_when_applicable() {
        let mut experiment = Experiment::default();
        experiment.add_ignore(|control, candidate| { candidate.value == 2 });

        let a = create_observation("a", 1);
        let b = create_observation("b", 2);
        let c = create_observation("c", 3);

        let result = ExperimentResult::new(
            &experiment,
            vec![a.clone(), b.clone(), c.clone()],
            0
        );

        assert!(result.has_mismatches());
        assert_eq!(vec![&c], result.mismatched());
        assert!(result.has_ignores());
        assert_eq!(vec![&b], result.ignored());
    }

    #[test]
    fn should_expose_experiment_name() {
        let experiment = Experiment::default();

        let a = create_observation("a", 1);
        let b = create_observation("b", 1);

        let result = ExperimentResult::new(
            &experiment,
            vec![a, b],
            0
        );

        assert_eq!(experiment.name, result.experiment_name);
    }

    #[test]
    fn should_expose_context_from_experiment() {
        let mut experiment = Experiment::default();
        experiment.add_context(Context::from_value(json!({"foo": "bar"})).unwrap());

        let a = create_observation("a", 1);
        let b = create_observation("b", 1);

        let result = ExperimentResult::new(
            &experiment,
            vec![a, b],
            0
        );

        assert_eq!(Context::from_value(json!({"foo": "bar"})).unwrap(), result.context);
    }

    fn create_observation<R: Clone + PartialEq + Serialize>(
        name: &'static str,
        value: R
    ) -> Observation<R> {
        return Observation::new(
            name.to_string(),
            "experiment".to_string(),
            value,
            None,
            1
        );
    }
}
