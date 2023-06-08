const REPLACE_CHAR: &str = "_";

#[cfg(target_os = "windows")]
const BLOCKED_CHARS: &str = "#%&{}\\$!'\":@<>*?/+`|=";

#[cfg(target_os = "linux")]
const BLOCKED_CHARS: &str = "/";

pub fn sanitize_title<S>(title: &S) -> String
where
    S: ToString + ?Sized,
{
    let title = title.to_string();
    let mut res = title.clone();

    title
        .chars()
        .map(|title_char| {
            BLOCKED_CHARS
                .chars()
                .filter(|blocked_char| blocked_char == &title_char)
                .for_each(|blocked_char| {
                    res = title.replace(blocked_char, REPLACE_CHAR);
                })
        })
        .for_each(|_| {});

    res
}
