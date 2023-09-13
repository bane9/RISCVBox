mod xmem;
use xmem::page_container::Xmem;

fn main() {
    let mut xmem = Xmem::new(1).unwrap();
    xmem.mark_rw().unwrap();
    xmem.mark_rx().unwrap();
    xmem.realloc(2).unwrap();
    xmem.mark_rw().unwrap();
    xmem.mark_rx().unwrap();
    xmem.dealloc();

    println!("done");
}
