const REPLACE_CHAR: char = '_';

#[cfg(target_os = "windows")]
const BLOCKED_CHARS: &str = "#%&{}\\$!'\":@<>*?/+`|=";

#[cfg(target_os = "linux")]
const BLOCKED_CHARS: &str = "/";

pub fn sanitize<S>(title: &S) -> String
where
    S: ToString + ?Sized,
{
    title
        .to_string()
        .chars()
        .map(|char| {
            if BLOCKED_CHARS.contains(char) {
                REPLACE_CHAR
            } else {
                char
            }
        })
        .collect()
}
