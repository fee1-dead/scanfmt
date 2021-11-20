# Scanfmt

This crate offers a macro for parsing text.

The macro accepts a format string, and the name of arguments to parse into.

The syntax of the format string literal is quite similar to the `format!` macro family:

```
format_string := text [ maybe_format text ] *
maybe_format := '{' '{' | '}' '}' | format
format := '{' [ argument ] [ ':' format_spec ] '}'
argument := integer | identifier

format_spec := 'o' | 'x' | 'X' | 'b'
```

In the above grammar, `text` must not contain any `'{'` or `'}'` characters.

## Usage

```rust
use scanfmt::{scanfmt, ScanError};
fn my_format(s: &str) -> Result<(u16, u32), ScanError> {
    let a;
    let b;
    scanfmt!(s, "a: {}, b: {}", a, b);
    Ok((a, b))
}
```

Note that scanfmt! requires the function to return a result to ensure that
variables are always initialized.