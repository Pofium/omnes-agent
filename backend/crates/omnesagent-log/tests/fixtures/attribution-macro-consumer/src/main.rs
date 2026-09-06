use omnesagent_log_attribution_macro_support::FixtureAttributable;

fn main() {
    let _span = omnesagent_log::attribution_span!(&FixtureAttributable);
}
