use std::str;
use std::slice;

// Define a function that is imported into the module.
// By default, the "env" namespace is used.
extern "C" {
    fn print_str(ptr: *const u8, len: usize);
}

// Define a string that is accessible within the wasm
// linear memory.
static HELLO: &'static str = "Hello, World!";

// Export a function named "hello_wasm". This can be called
// from the embedder!
#[no_mangle]
pub extern fn hello_wasm() {
    // Call the function we just imported and pass in
    // the offset of our string and its length as parameters.
    unsafe {
      print_str(HELLO.as_ptr(), HELLO.len());
    }
}


#[no_mangle]
pub extern fn hello_string_from_rust(ptr: i32, len: i32) {
    let slice = unsafe { slice::from_raw_parts(ptr as _, len as _) };
    let string_from_host = str::from_utf8(&slice).unwrap();
    let out_str = format!("Hello {}", string_from_host);
    unsafe {
      print_str(out_str.as_ptr(), out_str.len());
    }
}