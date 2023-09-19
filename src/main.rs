mod backend;
mod cpu;
mod frontend;
mod util;
mod xmem;

use frontend::exec_core::ExecCoreThreadPool;

fn main() {
    let rom = util::read_file("test.bin").unwrap();

    let exec_thread_pool = ExecCoreThreadPool::new(rom, 1);

    exec_thread_pool.join();
}
