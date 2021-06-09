# wasteland3saveeditor-rs
## compiling
you need a rust compiler , to get it, visit
https://rustup.rs/


## running 
1. build the project
```bash
cargo build --release
```
2. run the binary (it's being saved as `./target/release/wasteland3saveeditor`)

## example usage 
(for `unix-like` systems, should work on `windows` too, although `windows` has different syntax for setting environmental variables. You need `EDITOR` envvar to be your preferred text editor. Example on how to set it on linux is down here (`EDITOR=vim` before the command part)
```bash
cargo build --release # to build the binary
EDITOR=vim ./target/release/wasteland3saveeditor ~/.config/Wasteland3/Save\ Games/Quicksave\ 2/Quicksave\ 2.xml
