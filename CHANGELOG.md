# changelog for `handler_map`

## Pending
### Changes

- `HandlerMap` now has a lifetime parameter, and is not forced to `'static`

## `0.1.0` - 2018-09-19

Initial version!

- `HandlerMap` type:
  - `new`/`insert`/`remove`/`is_registered`/`val_is_registered`/`call`
- `BoxFn` implementation with `Opaque` ZST to erase function and argument type
