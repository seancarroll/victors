# TODO

List of TODOS

1. include a flag for raising/returning errors. See what scientist does but initial thought is that we might always want to make sure "control" runs 
2. Controlled vs Uncontrolled Experiment. I think this would provide a better API
3. Review structure of Observations and ExperimentResults
4. How to capture variables in closure
5. Capture exceptions and durations for observations
6. Avoiding clone to build experiment results
7. Cleaning Values
8. Comparing values 
9. Parallel computation 
10. OTEL publisher - will probably go into a separate lib dedicated to publishers
11. Should we just return serde json Value instead of a generic R from a behavior? It does need to technically be serializable so that it can be reported. - No for behavior and force serializable however for publisher...
12. Publishing Results - lets have publisher take in a ScientistValue (similar to serde JSON Value) so that we can
       remove the generic argument which should allow us to have a global exporter that callers can configure
13. Organize similar to opentelemetry with built in publishers
