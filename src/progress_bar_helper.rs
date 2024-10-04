use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Create a new progress bar with the given length.
/// The progress bar will be styled by default and will have a steady tick.
pub fn create_progress(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);

    pb.set_style(
        ProgressStyle::with_template("[{elapsed}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    pb.enable_steady_tick(Duration::from_millis(100));

    pb
}

/// Create a new spinner progress bar.
/// The progress bar will be styled by default and will have a steady tick.
pub fn create_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();

    pb.set_style(
        ProgressStyle::with_template("[{elapsed}] {spinner:.cyan/blue} {msg}")
            .unwrap_or(ProgressStyle::default_spinner())
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]),
    );

    pb.enable_steady_tick(Duration::from_millis(100));

    pb
}
