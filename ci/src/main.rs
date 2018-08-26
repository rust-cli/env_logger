mod task;
mod permute;

fn main() {
    let features = [
        "termcolor",
        "humantime",
        "atty",
        "regex",
    ];

    // Run a default build
    task::test(Default::default());

    // Run a set of permutations
    for features in permute::all(&features) {
        task::test(task::TestArgs {
            features,
            default_features: false,
            lib_only: true,
        });
    }
}
