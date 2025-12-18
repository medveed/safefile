pub const VERSION: u8 = 1;
pub const MAGIC: &[u8; 4] = b"SFIL";

#[macro_export]
macro_rules! progress_style {
    () => {
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} (ETA {eta})")
            .unwrap()
            .progress_chars("=> ")
    };
}

