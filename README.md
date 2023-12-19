[![crates.io](https://img.shields.io/crates/v/serde-sibor.svg)](https://crates.io/crates/serde-sibor)

# `serde-sibor`
#### `serde` implementation for the SiBOR binary format.

#### What is SiBOR?

SiBOR (**Si**mple **B**inary **O**bject **R**epresentation) is a binary format that is designed to
be simple to implement, fast to encode and decode, and relatively compact. In order to achieve
these goals, the number of features is kept to a minimum, and some types are not supported:

- SiBOR is not self-describing. The schema must be known in advance.
- SiBOR does not have a concept of "optional" fields. All fields must have a value.
- SiBOR does not support maps.
- SiBOR treats all signed integers, unsigned integers, and floats as 64-bit values.
- SiBOR encodes all unsigned integers using a variable-length encoding.
- SiBOR encodes all signed integers using a variable-length zigzag encoding.
- SiBOR encodes all floats using a 64-bit IEEE 754 encoding. The bits are treated as a u64 and encoded using the variable-length encoding.

SiBOR is meant to be used when you want a quick and dirty way to serialize and deserialize binary data of a known schema.
It does not have any built-in support for schema evolution, so such support must be implemented by the user.