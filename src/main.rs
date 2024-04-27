use std::process::Command;

fn main() {
    // let mut dockerps = Command::new("docker");
    // let mut dockerps = dockerps.arg("ps");
    let output = String::from_utf8(
        Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("json")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    println!("{:?}", output);
}
