use std::backtrace::Backtrace;
use thiserror::Error;

pub type VictorsResult<T> = Result<T, VictorsErrors>;

// TODO: not sure I like these structs but maybe should just go the tuple struct route

#[derive(Debug)]
pub struct BadBehavior {
    pub experiment: String,
    pub name: String,
    pub message: String,
}

#[derive(Debug)]
pub struct BehaviorMissing {
    pub experiment_name: String,
    pub name: String
}

#[derive(Debug)]
pub struct BehaviorNotUnique {
    pub experiment_name: String,
    pub name: String
}

#[derive(Debug)]
pub struct MismatchError {
    pub experiment_name: String,
    pub exception: Option<String>, // the associated class....not sure we need this,
    pub message: String, // TODO: probably dont need this.
    pub backtrace: Option<Backtrace>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum VictorsErrors {

    // TODO: I might want to have actual experiment to be part of enum variants but in order to
    // have experiment name would need to have defined struct (or maybe a trait fn)

    #[error("{}", .0.name)]
    BadBehavior(BadBehavior),

    #[error("{} missing {} behavior", .0.experiment_name, .0.name)]
    BehaviorMissing(BehaviorMissing),

    #[error("{} already has {} behavior", .0.experiment_name, .0.name)]
    BehaviorNotUnique(BehaviorNotUnique),

    #[error("experiment '{}' observations mismatched", .0.experiment_name)]
    MismatchError(MismatchError),


    // TODO: see if this is needed. looks like scientist doesnt use it anywhere
    #[error("{0} didn't return a value")]
    NoValue(String),
}

impl PartialEq for VictorsErrors {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                &VictorsErrors::BadBehavior(ref a),
                &VictorsErrors::BadBehavior(ref b)
            ) => {
                a.experiment == b.experiment
                    && a.name == b.name
                    && a.message == b.message
            },
            (
                &VictorsErrors::BehaviorMissing(ref a),
                &VictorsErrors::BehaviorMissing(ref b)
            ) => {
                a.experiment_name == b.experiment_name
                    && a.name == b.name
            },
            (
                &VictorsErrors::BehaviorNotUnique(ref a),
                &VictorsErrors::BehaviorNotUnique(ref b)
            ) => {
                a.experiment_name == b.experiment_name
                    && a.name == b.name
            },
            (
                &VictorsErrors::MismatchError(ref a),
                &VictorsErrors::MismatchError(ref b)
            ) => {
                a.experiment_name == b.experiment_name
                    && a.experiment_name == b.experiment_name
                    && a.message == b.message
            },
            (
                &VictorsErrors::NoValue(ref a),
                &VictorsErrors::NoValue(ref b)
            ) => {
                a == b
            },
            _ => false,
        }
    }
}
