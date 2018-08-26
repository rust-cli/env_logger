mod task;
mod permute;

use self::task::{test, TestArgs};
use self::permute::permute;

fn main() {
    let features = [
        "termcolor",
        "humantime",
        "atty",
        "regex",
    ];

    // Run a default build
    test(Default::default());

    // Run a set of permutations
    for features in permute(&features) {
        test(TestArgs {
            features,
            default_features: false,
            lib_only: true,
        });
    }
}
