use chrono::Local;
use env_logger::Builder;
use faster_ee::EE;
use std::io::Write;
use std::sync::Arc;

fn setup_logger() {
    Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}

fn main() {
    setup_logger();
    let my_ee = EE::new(String::from("tcp://*:8888"), None);
    let my_arc = Arc::new(my_ee);
    my_arc.run();
}
