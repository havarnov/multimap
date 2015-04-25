# Multimap implementation for Rust

This is a multimap implementation for Rust. Implemented as a thin wrapper around
std::collections::HashMap.

[Documentation](http://havarnov.github.io/multimap)

## Example

````rust
extern crate multimap;

use multimap::MultiMap;

fn main () {
    let mut map = MultiMap::new();

    map.insert("key1", 42);
    map.insert("key1", 1337);
    map.insert("key2", 2332);

    assert_eq!(map["key1"], 42);
    assert_eq!(map.get("key1"), Some(&42));
    assert_eq!(map.get_vec("key1"), Some(&vec![42, 1337]));
}
````

