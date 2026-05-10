const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_TAG: &str = match option_env!("GIT_TAG") {
    Some(v) => v,
    None => "unknown",
};
const GIT_COMMIT: &str = match option_env!("GIT_COMMIT") {
    Some(v) => v,
    None => "unknown",
};
const BUILD_DATE: &str = match option_env!("BUILD_DATE") {
    Some(v) => v,
    None => "unknown",
};

fn main() {
    println!("{PKG_NAME} {PKG_VERSION}");
    println!("git_tag:    {GIT_TAG}");
    println!("commit:     {GIT_COMMIT}");
    println!("build_date: {BUILD_DATE}");
}
