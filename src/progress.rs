use std::{borrow::Cow, fmt::Debug};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::info;

const PROGBAR_STYLE: &str = "{prefix:<50} [{bar:>50}] {msg} ({percent:.cyan}%)";
const PROGBAR_BAR_STYLE: &str = "█▓▒░";

pub struct Progress {
    master: MultiProgress,
    style: ProgressStyle,
}

impl Progress {
    pub fn new() -> Self {
        let style = ProgressStyle::with_template(PROGBAR_STYLE)
            .unwrap()
            .progress_chars(PROGBAR_BAR_STYLE); //"█▓▒░"
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

    pub fn add_custom_prog<P>(&self, total: u64, prefix: P, style: ProgressStyle) -> ProgressBar
    where
        P: Into<Cow<'static, str>> + Debug + Clone,
    {
        let new_prog = self.master.add(ProgressBar::new(total));
        new_prog.set_style(style.progress_chars(PROGBAR_BAR_STYLE));
        new_prog.tick();

        new_prog.set_prefix(prefix.clone());

        info!("Made new custom progress bar with prefix {:?}", prefix);
        new_prog
    }
}
