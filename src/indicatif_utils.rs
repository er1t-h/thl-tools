use indicatif::{MultiProgress, ProgressStyle};

pub trait IndicatifProgressIterExt: indicatif::ProgressIterator {
    fn in_multi_progress(self, multi_progress: &MultiProgress) -> Self;
    fn in_optional_multi_progress(self, multi_progress: Option<&MultiProgress>) -> Self {
        if let Some(multi_progress) = multi_progress {
            self.in_multi_progress(multi_progress)
        } else {
            self
        }
    }
}

impl<T: Iterator> IndicatifProgressIterExt for indicatif::ProgressBarIter<T> {
    fn in_multi_progress(mut self, multi_progress: &MultiProgress) -> Self {
        self.progress = multi_progress.add(self.progress);
        self
    }
}

#[allow(dead_code)]
pub fn default_bar_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise} < {eta_precise}] {bar} {pos:>7}/{len:7} {msg}")
        .unwrap()
}

pub fn default_bar_style_with_message_header(message_header: &str) -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(&format!("[{{elapsed_precise}} < {{eta_precise}}] {{bar}} {{pos:>7}}/{{len:7}} {message_header}: {{msg}}"))
        .unwrap()
}

pub fn default_spinner_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {spinner} {msg}")
        .unwrap()
}

pub fn default_spinner_style_with_message_header(message_header: &str) -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(&format!(
            "[{{elapsed_precise}}] {{spinner}} {message_header}: {{msg}}"
        ))
        .unwrap()
}
