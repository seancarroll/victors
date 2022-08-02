use crate::errors::VictorsResult;
use crate::experiment::{Experiment, UncontrolledExperiment};

pub struct Victor;

/// Helper that will instantiate and experiment and call run for you
impl Victor {

    /// Define and run a controlled experiment.
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `experiment_block` - Function to configure the experiment
    ///
    /// # Return
    /// Returns the calculated value of the control experiment or error
    pub fn conduct<F, R: Clone + PartialEq>(name: &'static str, experiment_block: F) -> VictorsResult<R>
        where
            F: Fn(&mut Experiment<'_, R>) -> VictorsResult<()>
    {
        let mut experiment = Experiment::new(name);
        experiment_block(&mut experiment)?;
        return experiment.run();
    }

    /// Define and run an uncontrolled experiment.
    /// Uncontrolled experiments contain only candidates and do not have a control.
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `return_candidate_result` - name of the candidate's result to return
    /// * `experiment_block` - Function to configure the experiment
    ///
    /// # Return
    /// Returns the calculated value of the named experiment or error
    pub fn conduct_uncontrolled<F, R: Clone + PartialEq>(
        name: &'static str,
        return_candidate_result: &'static str,
        experiment_block: F
    ) -> VictorsResult<R>
        where
            F: Fn(&mut UncontrolledExperiment<'_, R>) -> VictorsResult<()>
    {
        let mut experiment = UncontrolledExperiment::new(name);
        experiment_block(&mut experiment)?;
        return experiment.run(return_candidate_result);
    }
}
