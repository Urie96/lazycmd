use std::{cell::RefCell, rc::Rc};
use tokio::join;

#[derive(Debug)]
struct Time {
    hour: RefCell<u8>,
    minute: RefCell<u8>,
}

impl Time {
    pub fn set_time(&self, hour: u8, minute: u8) {
        *self.hour.borrow_mut() = hour;
        *self.minute.borrow_mut() = minute;
    }
}

async fn task_1(time: Rc<Time>) {
    time.set_time(11, 54);
    println!("Task 1: {:?}", time);
}

async fn task_2(time: Rc<Time>) {
    time.set_time(8, 12);
    println!("Task 2: {:?}", time);
}

// fn main() {
//     let mut runtime_builder = tokio::runtime::Builder::new_current_thread();
//     runtime_builder.enable_time();
//     let runtime = runtime_builder.build().unwrap();
//     runtime.block_on(async {
//         let time = Time {
//             hour: RefCell::new(0),
//             minute: RefCell::new(0),
//         };
//
//         let _ = join!(task_1(&time), task_2(&time));
//     });
// }

#[tokio::main]
async fn main() {
    let time = Time {
        hour: RefCell::new(0),
        minute: RefCell::new(0),
    };
    let time = Rc::new(time);
    let time2 = Rc::clone(&time);

    let _ = join!(task_1(time), task_2(time2));
}
