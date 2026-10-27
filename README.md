# xmlsec1-src

Source code of [xmlsec1](https://github.com/lsh123/xmlsec) (OpenSSL backend) and the logic to
build it as a static library. Meant to be used as a build-dependency of the
[`xmlsec`](https://github.com/zerocom38/rust-xmlsec) crate for its `vendored` feature, the same way
`openssl-src` is used by `openssl-sys`.

Only xmlsec itself is vendored. Its OpenSSL and libxml2 dependencies are linked by `openssl-sys`
and `libxml`; this crate only needs their headers:

- **OpenSSL**: `DEP_OPENSSL_INCLUDE` (set by `openssl-sys` for every crate depending on it), or
  `Build::openssl_include_dir`.
- **libxml2**: vcpkg on MSVC targets, pkg-config elsewhere, or `Build::libxml2_include_dir`.

## Usage

```toml
[features]
vendored = ["dep:xmlsec1-src"]

[dependencies]
openssl-sys = "0.9"
libxml = "0.3"

[build-dependencies]
xmlsec1-src = { version = "1", optional = true }
```

```rust
// build.rs
#[cfg(feature = "vendored")]
fn find_xmlsec() -> (Vec<PathBuf>, Vec<String>) {
    let artifacts = xmlsec1_src::Build::new().build();
    artifacts.print_cargo_metadata();

    let defines = artifacts
        .defines()
        .iter()
        .map(|(name, value)| match value {
            Some(v) => format!("-D{name}={v}"),
            None => format!("-D{name}"),
        })
        .collect();
    (vec![artifacts.include_dir().to_path_buf()], defines)
}

#[cfg(not(feature = "vendored"))]
fn find_xmlsec() -> (Vec<PathBuf>, Vec<String>) {
    // pkg-config / vcpkg lookup of a system xmlsec1
}
```

The returned include paths and defines are what bindgen needs. The defines (`XMLSEC_STATIC`,
`XMLSEC_CRYPTO_OPENSSL`, `XMLSEC_NO_XSLT`, ...) must match the ones the library was built with.

## Versioning

The crate version carries the bundled xmlsec version as build metadata, e.g. `1.0.0+1.3.13`.
To update xmlsec, move the `xmlsec` submodule and compare `SOURCES` in `src/lib.rs` with
`xmlsec/src/Makefile.am` and `xmlsec/src/openssl/Makefile.am`.

## Testing

```sh
git submodule update --init
cargo test -p testcrate
```
