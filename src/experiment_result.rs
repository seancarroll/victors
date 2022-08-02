use std::collections::HashMap;
use serde_json::Value;
use crate::experiment::Experiment;
use crate::observation::Observation;

// TODO: given this should be immutable remove pub from fields
/// The immutable result of running an experiment.
#[derive(Clone, PartialEq)]
pub struct ExperimentResult<R: Clone + PartialEq> {
    // pub experiment: &'a dyn Experiment,
    // TODO: ugh...this would need to change in order to use controlled/uncontrolled
    // could be a trait object I suppose and see how well that works
    // pub experiment: &'a Experiment<R>,
    pub experiment_name: String,
    pub observations: Vec<Observation<R>>,
    pub context: HashMap<String, Value>,
    // pub candidates: Vec<Observation<R>>,
    // pub control: &'a Observation<R>,
    // TODO: I dont love using "control" here as its abusing the name for uncontrolled experiments
    // need to find a better term. Maybe primary behavior
    pub control_index: usize,
    // pub ignored: Vec<&'a Observation<R>>,
    // pub mismatched: Vec<&'a Observation<R>>,

    mismatched_indexes: Vec<usize>,
    ignored_indexes: Vec<usize>
}

impl<'a, R: Clone + PartialEq> ExperimentResult<R> {
    // pub fn new(
    //     experiment: &'a Experiment<R>,
    //     observations: Vec<Observation<R>>,
    //     control: &'a Observation<R>
    // ) -> Self {
    //     let (mismatched_observations, ignored_observations)
    //         = ExperimentResult::evaluate_candidates(experiment, &observations, control);
    //
    //     Self {
    //         candidates: vec![], // observations.rem,
    //         experiment,
    //         observations,
    //         control,
    //         mismatched: mismatched_observations,
    //         ignored: ignored_observations,
    //     }
    // }

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
        let (mismatched_indexes, ignored_indexes)
            = ExperimentResult::evaluate_candidates(experiment, &observations, control_index);
        println!("experiment context is [{:?}]", &experiment.context);
        Self {
            experiment_name: experiment.name.to_string(),
            observations,
            context: experiment.context.clone(),
            control_index,
            mismatched_indexes,
            ignored_indexes,
        }
    }

    // TODO: rename as i dont like "control"
    /// Returns the control observation
    pub fn control(&self) -> Option<&Observation<R>> {
        return self.observations.get(self.control_index);
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

    pub fn matched(&self) -> bool {
        return self.mismatched_indexes.is_empty() && !self.has_ignores();
    }

    pub fn has_matched(&self) -> bool {
        return !self.mismatched_indexes.is_empty();
    }

    pub fn has_ignores(&self) -> bool {
        return !self.ignored_indexes.is_empty();
    }

    // pub fn matched(&self) -> bool {
    //     return self.mismatched.is_empty() && !self.has_ignores();
    // }
    //
    // pub fn has_matched(&self) -> bool {
    //     return !self.mismatched.is_empty();
    // }
    //
    // pub fn has_ignores(&self) -> bool {
    //     return !self.ignored.is_empty();
    // }

    // Internal: evaluate the candidates to find mismatched and ignored results
    //
    // Sets @ignored and @mismatched with the ignored and mismatched candidates.
    // /// Evaluate the candidates to find mismatched and ignored results.
    // fn evaluate_candidates(
    //     experiment: &'a Experiment<R>,
    //     candidates: &'a Vec<Observation<R>>,
    //     control: &'a Observation<R>
    // ) -> (Vec<&'a Observation<R>>, Vec<&'a Observation<R>>) {
    //     let mut mismatched = vec![];
    //     let mut ignored = vec![];
    //     for candidate in candidates.into_iter() {
    //         let is_equivalent = experiment.observations_are_equivalent(&control, candidate);
    //         if !is_equivalent {
    //             let ignore = experiment.ignore_mismatch_observation(&control, candidate);
    //             if ignore {
    //                 ignored.push(candidate);
    //             } else {
    //                 mismatched.push(candidate);
    //             }
    //         }
    //     }
    //
    //     return (mismatched, ignored);
    // }

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
