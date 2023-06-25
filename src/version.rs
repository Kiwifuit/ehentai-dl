const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROGRAM_COMMIT: &str = include_str!("../.git/refs/heads/master");
const COMMIT_HASH_LENGTH: usize = 8;

pub fn get_version() -> String {
    let features = get_features();

    format!(
        "{} v{}@{} {}",
        PROGRAM_NAME,
        PROGRAM_VERSION,
        &PROGRAM_COMMIT[..COMMIT_HASH_LENGTH],
        if !features.is_empty() {
            format!("w/{}", features.join(","))
        } else {
            String::new()
        }
    )
    .trim_end()
    .to_string()
}

#[allow(unused_mut)]
pub fn get_features() -> Vec<String> {
    let mut res = vec![];

    #[cfg(feature = "aniyomi")]
    res.push("aniyomi");
    #[cfg(feature = "zip")]
    res.push("zip");
    #[cfg(feature = "config")]
    res.push("config");
    #[cfg(feature = "cli")]
    res.push("cli");

    res.iter().map(|i: &&str| String::from(*i)).collect()
}
