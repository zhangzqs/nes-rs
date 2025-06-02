use std::{cell::RefCell, rc::Rc};

use nes_base::{BusAdapter, CPU};

struct MainBoard {
    cpu: Rc<RefCell<dyn CPU>>,
    ram: Rc<RefCell<dyn BusAdapter>>, // 2KB of RAM
}
