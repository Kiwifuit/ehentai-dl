use std::{borrow::Cow, fmt::Debug};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::info;

const PROGBAR_STYLE: &str = "{prefix} [{wide_bar}] {msg} ({percent:.cyan}%)";

pub struct Progress {
    master: MultiProgress,
    style: ProgressStyle,
}

impl Progress {
    pub fn new() -> Self {
        let style = ProgressStyle::with_template(PROGBAR_STYLE)
            .unwrap()
            .progress_chars("█▓▒░"); //"█▓▒░"
        let master = MultiProgress::new();

        info!("Master Progress Bar created");
        Self { master, style }
    }

    pub fn add_prog<P>(&self, total: u64, prefix: P) -> ProgressBar
    where
        P: Into<Cow<'static, str>> + Debug + Clone,
    {
        let new_prog = self.master.add(ProgressBar::new(total));
        new_prog.set_style(self.style.clone());
        new_prog.tick();

        new_prog.set_prefix(prefix.clone());

        info!("Made new progress bar with prefix {:?}", prefix);
        new_prog
    }
}
