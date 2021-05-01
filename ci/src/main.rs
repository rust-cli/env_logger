mod permute;
mod task;

fn main() {
    let features = ["termcolor", "humantime", "atty", "regex"];

    // Run a default build
    if !task::test(Default::default()) {
        panic!("default test execution failed");
    }

    // Run a build with no features
    if !task::test(task::TestArgs {
        default_features: false,
        ..Default::default()
    }) {
        panic!("default test execution failed");
    }

    // Run a set of permutations
    let failed = permute::all(&features)
        .into_iter()
        .filter(|features| {
            !task::test(task::TestArgs {
                features: features.clone(),
                default_features: false,
                lib_only: true,
            })
        })
        .collect::<Vec<_>>();

    if !failed.is_empty() {
        for failed in failed {
            eprintln!("FAIL: {:?}", failed);
        }

        panic!("test execution failed");
    } else {
        println!("test execution succeeded");
    }
}
