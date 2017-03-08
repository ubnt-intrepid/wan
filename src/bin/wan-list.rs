extern crate wan;

fn main() {
  for info in wan::get_compiler_info().unwrap() {
    println!("{}, \"{}\"", info.name(), info.display_compile_command());
  }
}
