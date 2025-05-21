use indicatif::MultiProgress;

pub trait IndicatifProgressExt {
    fn in_multi_progress(self, multi_progress: &MultiProgress) -> Self;
    fn in_optional_multi_progress(self, multi_progress: Option<&MultiProgress>) -> Self
    where
        Self: Sized,
    {
        if let Some(multi_progress) = multi_progress {
            self.in_multi_progress(multi_progress)
        } else {
            self
        }
    }
}

impl<T> IndicatifProgressExt for indicatif::ProgressBarIter<T> {
    fn in_multi_progress(mut self, multi_progress: &MultiProgress) -> Self {
        self.progress = multi_progress.add(self.progress);
        self
    }
}
impl IndicatifProgressExt for indicatif::ProgressBar {
    fn in_multi_progress(self, multi_progress: &MultiProgress) -> Self {
        multi_progress.add(self)
    }
}

#[allow(dead_code)]
mod default_styles {
    use indicatif::ProgressStyle;

    pub fn byte_bar_style() -> ProgressStyle {
        ProgressStyle::default_bar()
        .template("[{elapsed_precise} < {eta_precise} | {percent_precise}% | {decimal_bytes_per_sec}] {bar} {decimal_bytes:>12}/{decimal_total_bytes:12} {msg}")
        .unwrap()
    }

    pub fn byte_bar_style_with_message_header(message_header: &str) -> ProgressStyle {
        ProgressStyle::default_bar()
        .template(&format!("[{{elapsed_precise}} < {{eta_precise}} | {{percent_precise}}% | {{decimal_bytes_per_sec}}] {{bar}} {{decimal_bytes:>12}}/{{decimal_total_bytes:12}} {message_header}: {{msg}}"))
        .unwrap()
    }

    pub fn default_bar_style() -> ProgressStyle {
        ProgressStyle::default_bar()
            .template("[{elapsed_precise} < {eta_precise} | {percent_precise}% | {per_sec:11}] {bar} {pos:>7}/{len:7} {msg}")
            .unwrap()
    }

    pub fn default_bar_style_with_message_header(message_header: &str) -> ProgressStyle {
        ProgressStyle::default_bar()
        .template(&format!("[{{elapsed_precise}} < {{eta_precise}} | {{percent_precise}}% | {{per_sec:11}}] {{bar}} {{pos:>7}}/{{len:7}} {message_header}: {{msg}}"))
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
}

pub use default_styles::*;
