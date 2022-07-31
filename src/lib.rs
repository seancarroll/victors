#![doc(html_root_url = "https://docs.rs/victors")]
#![doc(issue_tracker_base_url = "https://github.com/seancarroll/victors/issues/")]
#![cfg_attr(docsrs, deny(broken_intra_doc_links))]
#![feature(backtrace)]

mod errors;
mod experiment;
mod experiment_result;
mod observation;
mod victor;
mod result_publisher;

#[cfg(test)]
mod tests {
    use crate::errors::{BehaviorNotUnique, VictorsErrors, VictorsResult};
    use crate::experiment::Experiment;
    use crate::victor::Victor;

    #[test]
    fn it_works() {
        let mut experiment = Experiment::default();
        experiment.control(|| { println!("control...")}).expect("control shouldnt fail");
        experiment.candidate("", || { println!("candidate...") }).expect("candidate shouldnt fail");

        let result = experiment.run();
    }

    #[test]
    fn should_be_able_to_specify_control_name() {
        let mut experiment = Experiment::new("custom control");
        experiment.control(|| { println!("control...")}).expect("control shouldnt fail");

        assert_eq!("custom control", experiment.name);
    }

    // TODO: custom enabled fn
    // TODO: pass context
    // TODO: publish
    // TODO: clean values
    // TODO: run_if
    // TODO: parallel
    // TODO: before_run setup


    // it "runs the named test instead of the control" do

    // #[test]
    // fn should_allow_to_build_experiment_fluently() -> VictorsResult<()> {
    //     let experiment = Victor::default()?
    // }

    #[test]
    fn should_be_able_to_create_and_run_experiment_via_victor() {
        let r = Victor::conduct("conduct test", |experiment| {
            // To fix the error below means we need to return Ok(()) at the end
            // cannot use the `?` operator in a closure that returns `()`
            // I dont love it but I suppose thats part of the rust idioms
            experiment.control(|| { 1 })?;
            experiment.candidate("candidate", || { 2 })?;
            Ok(())
        });

        assert_eq!(Some(1), r.ok());
    }

    #[test]
    fn should_be_able_to_create_and_run_uncontrolled_experiment_via_victor() {
        let r = Victor::conduct_uncontrolled("conduct test", "second", |experiment| {
            // To fix the error below means we need to return Ok(()) at the end
            // cannot use the `?` operator in a closure that returns `()`
            // I dont love it but I suppose thats part of the rust idioms
            experiment.candidate("first", || { 1 })?;
            experiment.candidate("second", || { 2 })?;
            Ok(())
        });

        assert_eq!(Some(2), r.ok());
    }

    #[test]
    fn should_return_non_unique_error_when_multiple_candidates_are_registered_with_same_name() {
        let mut experiment = Experiment::default();
        experiment.control(|| { println!("control...")}).expect("control shouldnt fail");
        experiment.candidate("candidate", || { println!("candidate...") }).expect("candidate shouldnt fail");
        let result = experiment.candidate("candidate", || { println!("second candidate...") });
        let expected = VictorsErrors::BehaviorNotUnique(BehaviorNotUnique {
            experiment_name: "experiment".to_string(),
            name: "candidate".to_string()
        });
        assert_eq!(expected, result.unwrap_err());
    }


    // ignore ignore_mismatched_observation tests
    // TODO: does not ignore an observation if no ignores are configured
    // TODO: calls a configured ignore block with the given observed values
    // TODO: calls multiple ignore blocks to see if any match
    // TODO: only calls ignore blocks until one matches
    // TODO: reports exceptions raised in an ignore block and returns false
    // TODO: skips ignore blocks that raise and tests any remaining blocks if an exception is swallowed
}
