
// shared default scientist / victor
// static readonly Lazy<Scientist> _sharedScientist = new Lazy<Scientist>(CreateSharedInstance);
// static Scientist CreateSharedInstance() => new SharedScientist(ResultPublisher);

// Scientist::Default.new "widget-permissions"

use crate::errors::VictorsResult;
use crate::experiment::Experiment;

// Creating an experiment is wordy, but when you include the Scientist module, the science helper
// will instantiate an experiment and call run for you:
// science "widget-permissions" do |experiment|
//    experiment.use { model.check_user(user).valid? } # old way
//    experiment.try { user.can?(:read, model) } # new way
// end # returns the control value
pub struct Victor;

impl Victor {

    /// Define and run an experiment.
    ///
    /// # Arguments
    /// * `name` - the name of the experiment
    /// * `experiment_block` - Function that allows configuring the experiment
    ///
    /// # Return
    /// Returns the calculated value of the control experiment or error
    pub fn conduct<F, R: Clone>(name: &'static str, experiment_block: F) -> VictorsResult<R>
        where
            F: Fn(&mut Experiment<R>) -> VictorsResult<()>
    {
        let mut experiment = Experiment::new(name);
        experiment_block(&mut experiment)?;
        return experiment.run();
    }

}
