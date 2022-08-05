#![doc(html_root_url = "https://docs.rs/victors")]
#![doc(issue_tracker_base_url = "https://github.com/seancarroll/victors/issues/")]
#![cfg_attr(docsrs, deny(broken_intra_doc_links))]
#![feature(backtrace)]
#![deny(elided_lifetimes_in_paths)]

pub mod context;
pub mod errors;
pub mod experiment;
pub mod experiment_result;
pub mod observation;
pub mod result_publisher;
pub mod victor;

// TODO: can i use *?
// https://github.com/SeaQL/sea-orm/blob/master/src/lib.rs
pub use crate::{
    context::Context,
    experiment::{Experiment, UncontrolledExperiment},
    experiment_result::ExperimentResult,
    observation::Observation,
    result_publisher::Publisher,
};

#[cfg(test)]
mod tests {
    use std::{
        borrow::Borrow,
        cell::{Ref, RefCell},
        collections::HashMap,
        pin::Pin,
        rc::Rc,
    };

    use serde_json::{json, Value};

    use crate::{
        context::Context,
        errors::{BehaviorMissing, BehaviorNotUnique, VictorsErrors, VictorsResult},
        experiment::Experiment,
        experiment_result::ExperimentResult,
        observation::Observation,
        result_publisher::InMemoryPublisher,
        victor::Victor,
    };

    #[test]
    fn it_works() {
        let mut experiment = Experiment::default();
        experiment.control(|| println!("control...")).unwrap();
        experiment.candidate(|| println!("candidate...")).unwrap();

        experiment.run().unwrap();
    }

    #[test]
    fn should_be_able_to_specify_control_name() {
        let mut experiment = Experiment::new("custom control");
        experiment.control(|| println!("control...")).unwrap();

        assert_eq!("custom control", experiment.name);
    }

    #[test]
    fn should_return_error_when_controlled_experiment_has_no_control_behavior_defined() {
        let mut experiment = Experiment::default();
        experiment.candidate(|| 1).unwrap();
        let result = experiment.run();
        let expected = VictorsErrors::BehaviorMissing(BehaviorMissing {
            experiment_name: "experiment".to_string(),
            name: "control".to_string(),
        });
        assert_eq!(expected, result.unwrap_err());
    }

    // TODO: custom enabled fn

    #[test]
    fn should_allow_to_init_experiment_with_context() {
        let r: RefCell<Option<ExperimentResult<u8>>> = RefCell::new(None);

        let mut experiment = Experiment::new_with_context(
            "experiment",
            Context::from_value(json!({"message": "hello world"})).unwrap(),
        );
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| 1).unwrap();
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.replace(Some(result.clone()));
        }));

        experiment.run().unwrap();

        assert_eq!(
            r.take().unwrap().context.get(&"message".to_string()),
            Some(&Value::String("hello world".to_string()))
        );
    }

    #[test]
    fn should_allow_custom_context_to_be_passed_in() {
        let r: RefCell<Option<ExperimentResult<u8>>> = RefCell::new(None);

        let mut experiment = Experiment::default();
        experiment.add_context(Context::from_value(json!({"message": "hello world"})).unwrap());
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| 1).unwrap();
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.replace(Some(result.clone()));
        }));

        experiment.run().unwrap();

        assert_eq!(
            r.take().unwrap().context.get(&"message".to_string()),
            Some(&Value::String("hello world".to_string()))
        );
    }

    // TODO: publish
    // TODO: clean values

    // TODO: run_if - run "experiment" means candidates but not sure if we really need to change
    // the language or not
    #[test]
    fn should_not_run_candidates_when_run_if_returns_false() {
        let called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| {
            called.replace(true);
            1
        }).unwrap();
        experiment.run_if(|| false);

        experiment.run().unwrap();

        assert!(!called.take());
    }

    #[test]
    fn should_run_candidates_when_run_if_returns_true() {
        let called = RefCell::new(false);

        let mut experiment = Experiment::default();
        experiment.control(|| 1).unwrap();
        experiment.candidate(|| {
            called.replace(true);
            1
        }).unwrap();
        experiment.run_if(|| true);

        experiment.run().unwrap();

        assert!(called.take());
    }

    // TODO: parallel
    // TODO: before_run setup

    #[test]
    fn should_be_able_to_create_and_run_experiment_via_victor() {
        let r = Victor::conduct("conduct test", |experiment| {
            // To fix the error below means we need to return Ok(()) at the end
            // cannot use the `?` operator in a closure that returns `()`
            // I dont love it but I suppose thats part of the rust idioms
            experiment.control(|| 1)?;
            experiment.candidate(|| 2)?;
            Ok(())
        });

        assert_eq!(Some(1), r.ok());
    }

    #[test]
    fn should_be_able_to_create_and_run_uncontrolled_experiment_via_victor() {
        let r = Victor::conduct_uncontrolled("uncontrolled test", "second", |experiment| {
            // To fix the error below means we need to return Ok(()) at the end
            // cannot use the `?` operator in a closure that returns `()`
            // I dont love it but I suppose thats part of the rust idioms
            experiment.candidate("first", || 1)?;
            experiment.candidate("second", || 2)?;
            Ok(())
        });

        assert_eq!(Some(2), r.ok());
    }

    #[test]
    fn should_return_non_unique_error_when_multiple_candidates_are_registered_with_same_name() {
        let mut experiment = Experiment::default();
        experiment.control(|| println!("control...")).unwrap();
        experiment.candidate(|| println!("candidate...")).unwrap();
        let result = experiment.candidate(|| println!("second candidate..."));
        let expected = VictorsErrors::BehaviorNotUnique(BehaviorNotUnique {
            experiment_name: "experiment".to_string(),
            name: "candidate".to_string(),
        });
        assert_eq!(expected, result.unwrap_err());
    }

    // ignore ignore_mismatched_observation tests
    // TODO: does not ignore an observation if no ignores are configured
    // TODO: calls a configured ignore block with the given observed values
    #[test]
    fn should_call_ignore_blocks_until_match() {
        // TODO: using RefCell seems like a hack. I can't figure out how to just mutate a boolean
        // captured variable. All three types (Fn, FnOnce, FnMut) are problematic for various reasons
        let (called_one, called_two, called_three) = (RefCell::new(false), RefCell::new(false), RefCell::new(false));
        // let mut called_one = false;

        let mut experiment = Experiment::default();
        experiment.add_ignore(|_a, _b| {
            called_one.replace(true);
            false
        });
        experiment.add_ignore(|_a, _b| {
            called_two.replace(true);
            true
        });
        experiment.add_ignore(|_a, _b| {
            called_three.replace(true);
            false
        });

        let a = create_observation("a");
        let b = create_observation("b");

        assert!(experiment.ignore_mismatch_observation(&a, &b));
        assert!(called_one.take());
        assert!(called_two.take());
        assert!(!called_three.take());
    }

    // TODO: can't be run without a control behavior  --> this is at least one behavior
    // TODO: runs other behaviors but always returns the control
    // TODO: complains about duplicate behavior names
    // TODO: swallows exceptions raised by candidate behaviors
    // TODO: shuffles behaviors before running
    // TODO: re-raises exceptions raised during publish by default
    // TODO: reports publishing errors
    // TODO: publishes results
    // TODO: does not publish results when there is only a control value
    // TODO: compares results with a comparator block if provided
    // TODO: compares errors with an error comparator block if provided
    #[test]
    fn should_compare_results_with_comparator_when_provided() {
        #[derive(Clone, PartialEq)]
        struct TestResult {
            count: u8,
            message: &'static str,
        }

        let r: RefCell<Option<ExperimentResult<TestResult>>> = RefCell::new(None);

        let mut experiment = Experiment::default();
        experiment.control(|| TestResult {
            count: 1,
            message: "control msg",
        }).unwrap();
        experiment.candidate(|| TestResult {
            count: 1,
            message: "candidate msg",
        }).unwrap();
        experiment.comparator(|a, b| a.count == b.count);
        experiment.result_publisher(InMemoryPublisher::new(|result| {
            r.replace(Some(result.clone()));
        }));

        let value = experiment.run().unwrap();
        assert_eq!(1, value.count);
        assert_eq!("control msg", value.message);
        assert!(r.take().unwrap().matched());
    }

    // TODO: knows how to compare two experiments
    // TODO: uses a compare block to determine if observations are equivalent
    // TODO: reports errors in a compare block
    // TODO: reports errors in the enabled? method
    // TODO: reports errors in a run_if block
    // TODO: returns the given value when no clean block is configured
    // TODO: calls the configured clean block with a value when configured
    // TODO: reports an error and returns the original value when an error is raised in a clean block

    // TODO: calls multiple ignore blocks to see if any match
    // TODO: only calls ignore blocks until one matches
    // TODO: reports exceptions raised in an ignore block and returns false
    // TODO: skips ignore blocks that raise and tests any remaining blocks if an exception is swallowed

    // TODO: raising on mismatches
    // TODO: "raises when there is a mismatch if raise on mismatches is enabled"
    // TODO: "cleans values when raising on observation mismatch"
    // TODO: "doesn't raise when there is a mismatch if raise on mismatches is disabled"
    // TODO: "raises a mismatch error if the control raises and candidate doesn't"
    // TODO: "raises a mismatch error if the candidate raises and the control doesn't"

    // TODO: can be marshaled

    // TODO: raise_on_mismatches
    // TODO: raises when there is a mismatch if the experiment instance's raise on mismatches is enabled
    // TODO: doesn't raise when there is a mismatch if the experiment instance's raise on mismatches is disabled

    // TODO: MismatchError
    // TODO: has the name of the experiment
    // TODO: includes the experiments' results
    // TODO: formats nicely as a string
    // TODO: includes the backtrace when an observation raises

    // TODO: before run block
    // TODO: does not run when an experiment is disabled

    fn create_observation(name: &'static str) -> Observation<u8> {
        return Observation::new(name.to_string(), "experiment".to_string(), 1, None, 1);
    }
}
