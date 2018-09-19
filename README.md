# `handler_map`

*like `AnyMap`, but with functions instead of values*

This crate began with an idle thought: *"Crates like `AnyMap` let you store one value of each type.
What would it take to instead store a function that took that type, like a message handler?"* What
came out was this.

The basic idea is that you start with a message type, and a function that receives it by-value:

```rust
struct MyMessage;
fn handle_msg(_: MyMessage) {
    println!("Got your message!");
}
```

Then, take one of these `HandlerMap`s, and hand it the handler:

```rust
let mut map = HandlerMap::new();
map.insert(handle_msg);
```

This registers that type in the handler so you can call it later:

```rust
map.call(MyMessage);

// console prints "Got your message!"
```
