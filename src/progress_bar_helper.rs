use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressBarHelper {}

impl ProgressBarHelper {
    /// Create a new progress bar with the given length.
    /// The progress bar will be styled by default and will have a steady tick.
    pub fn new(length: u64) -> ProgressBar {
        let pb = ProgressBar::new(length);

        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
                .unwrap_or(ProgressStyle::default_bar())
                .progress_chars("#>-"),
        );

        pb.enable_steady_tick(Duration::from_millis(100));

        pb
    }
}
