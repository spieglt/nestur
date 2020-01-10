use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let mapper = Rc::new(RefCell::new(Mmc1{a:32}));
    
    let ppu = Ppu{mapper: mapper.clone()};
    let cpu = Cpu{mapper: mapper.clone()};
    println!("{}", cpu.mapper.borrow_mut().doit());
    println!("{}", ppu.mapper.borrow_mut().doit());
}

struct Mmc1 {
    a: u8,
}

trait Mapper {
    fn doit(&mut self) -> u8;
}

impl Mapper for Mmc1 {
    fn doit(&mut self) -> u8 {
        self.a += 1;
        self.a
    }
}

struct Ppu {
    mapper: Rc<RefCell<dyn Mapper>>,
}

struct Cpu {
    mapper: Rc<RefCell<dyn Mapper>>,
}
