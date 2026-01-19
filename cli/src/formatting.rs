#[macro_export]
macro_rules! table_row {
    ($label:expr, $value:expr) => {
        println!("{:10} {}", $label.yellow(), $value);
    };
}

#[macro_export]
macro_rules! avg_speed {
    ($size:expr, $ms:expr) => {
        if $ms == 0 {
            "N/A".to_string()
        } else {
            let speed = ($size as u128 * 1000) / $ms;
            utils::bytes_to_human_readable(speed as usize) + "/s"
        }
    };
    () => {};
}