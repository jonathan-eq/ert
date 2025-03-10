use std::sync::Arc;

use faster_ee::EE;

fn main() {
    let my_ee = EE::new(
        String::from("tcp://*:8889"),
        None,
        String::from("jonak_ensemble"),
    );
    let my_arc = Arc::new(my_ee);
    my_arc.run();
}
