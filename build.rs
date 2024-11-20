use std::process::Command;

fn main() {
  if std::env::var("SKIP_FRONTEND_BUILD").is_err() {
    Command::new("yarn")
      .arg("build")
      .current_dir("archive-frontend")
      .status()
      .unwrap_or_else(|e| panic!("Failed to run `yarn build` {}", e));
  } else {
      println!("Skipping `yarn build`");
  }
}