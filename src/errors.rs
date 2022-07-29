use thiserror::Error;

pub type VictorsResult<T> = Result<T, VictorsErrors>;

#[non_exhaustive]
#[derive(Error, Debug, PartialEq)]
pub enum VictorsErrors {

    // TODO: I might want to have actual experiment to be part of enum variants but in order to
    // have experiment name would need to have defined struct (or maybe a trait fn)

    #[error("{message}")]
    BadBehavior {
        experiment: String,
        name: String,
        message: String,
    },

    #[error("{experiment_name} missing {name} behavior")]
    BehaviorMissing { experiment_name: String, name: String },

    #[error("{experiment_name} already has {name} behavior")]
    BehaviorNotUnique { experiment_name: String, name: String },

    #[error("{observation_name} didn't return a value")]
    NoValue {observation_name: String},
}
