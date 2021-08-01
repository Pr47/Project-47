use pr47::vm::al31f::compiled::CompiledProgram;
use pr47::vm::al31f::VMThread;
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;
use pr47::vm::al31f::executor::{create_vm_main_thread, vm_thread_run_function};
use pr47::vm::test::fibonacci_program;
use pr47::util::async_utils::block_on_future;
use pr47::data::Value;
use pr47::data::exception::Exception;
use std::time::Instant;

fn bench_fibonacci_call() {
    async fn run_fib35() {
        let program: CompiledProgram<DefaultAlloc> = fibonacci_program();
        let alloc: DefaultAlloc = DefaultAlloc::new();
        let mut vm_thread: VMThread<DefaultAlloc> = create_vm_main_thread(alloc, &program).await;
        let result: Result<Vec<Value>, Exception> = unsafe {
            vm_thread_run_function(&mut vm_thread, 0, &[Value::new_int(35)]).await
        };
        if let Err(_) = result {
            panic!("holy shit");
        }
    }

    block_on_future(run_fib35())
}

fn main() {
    bench_fibonacci_call();
}