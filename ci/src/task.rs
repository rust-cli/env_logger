use std::collections::BTreeSet;
use std::process::{
    Command,
    Stdio,
};

pub type Feature = &'static str;

pub struct TestArgs {
    pub features: BTreeSet<Feature>,
    pub default_features: bool,
    pub lib_only: bool,
}

impl Default for TestArgs {
    fn default() -> Self {
        TestArgs {
            features: BTreeSet::new(),
            default_features: true,
            lib_only: false,
        }
    }
}

impl TestArgs {
    fn features_string(&self) -> Option<String> {
        if self.features.len() == 0 {
            return None;
        }

        Some(self.features.iter().map(|f| *f).collect::<Vec<_>>().join(" "))
    }
}

pub fn test(args: TestArgs) {
    let features = args.features_string();

    let mut command = Command::new("cargo");
        
    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("test")
        .arg("--verbose");

    if !args.default_features {
        command.arg("--no-default-features");
    }

    if args.lib_only {
        command.arg("--lib");
    }

    if let Some(ref features) = features {
        command.args(&["--features", features]);
    }

    println!("running {:?}", command);
    
    let status = command
        .status()
        .expect("Failed to execute command");

    if !status.success() {
        eprintln!("test execution failed for features: {}", features.as_ref().map(AsRef::as_ref).unwrap_or(""));
    }
}
