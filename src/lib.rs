//! Vendored build of [xmlsec1](https://github.com/lsh123/xmlsec) with the OpenSSL backend.
//!
//! This crate is intended to be used as a build-dependency of a `-sys` style crate,
//! similar to how `openssl-src` is used by `openssl-sys`:
//!
//! ```no_run
//! // build.rs
//! let artifacts = xmlsec1_src::Build::new().build();
//! artifacts.print_cargo_metadata();
//! ```
//!
//! OpenSSL and libxml2 are not vendored; only their headers are needed to compile xmlsec,
//! linking them is left to `openssl-sys` and `libxml`.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Name of the static library produced by [`Build::build`].
const LIB_NAME: &str = "xmlsec1";

const SOURCES: &[&str] = &[
    "app.c",
    "base64.c",
    "bn.c",
    "buffer.c",
    "c14n.c",
    "dl.c",
    "enveloped.c",
    "errors.c",
    "io.c",
    "keyinfo.c",
    "keys.c",
    "keysdata.c",
    "keysdata_helpers.c",
    "keysmngr.c",
    "kw_helpers.c",
    "list.c",
    "membuf.c",
    "nodeset.c",
    "parser.c",
    "relationship.c",
    "strings.c",
    "templates.c",
    "transform_helpers.c",
    "transforms.c",
    "xmldsig.c",
    "xmlenc.c",
    "xmlsec.c",
    "xmltree.c",
    "xpath.c",
    "xslt.c",
    "x509_helpers.c",
    "openssl/app.c",
    "openssl/ciphers.c",
    "openssl/crypto.c",
    "openssl/der_encoded_value.c",
    "openssl/digests.c",
    "openssl/evp.c",
    "openssl/hmac.c",
    "openssl/kdf.c",
    "openssl/key_agrmnt.c",
    "openssl/key_encapsulation.c",
    "openssl/keysstore.c",
    "openssl/kt_rsa.c",
    "openssl/kw_rfc_3394.c",
    "openssl/kw_des.c",
    "openssl/signatures.c",
    "openssl/signatures_legacy.c",
    "openssl/symkeys.c",
    "openssl/x509.c",
    "openssl/x509vfy.c",
];

/// Preprocessor defines the library is built with. Consumers must use the same
/// defines when compiling against (or generating bindings for) the headers.
const DEFINES: &[(&str, Option<&str>)] = &[
    ("XMLSEC_STATIC", Some("1")),
    ("XMLSEC_CRYPTO_OPENSSL", Some("1")),
    ("XMLSEC_NO_CRYPTO_DYNAMIC_LOADING", Some("1")),
    ("XMLSEC_DEFAULT_CRYPTO", Some("\"openssl\"")),
    ("XMLSEC_NO_XSLT", Some("1")),
];

/// Directory containing the bundled xmlsec sources.
pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("xmlsec")
}

/// Version of the bundled xmlsec sources, e.g. `"1.3.13"`.
pub fn version() -> String {
    let configure_ac = source_dir().join("configure.ac");
    let content = fs::read_to_string(&configure_ac).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {e} (submodule not initialized?)",
            configure_ac.display()
        )
    });

    // The version is defined in AC_INIT([xmlsec1],[1.3.13],[...]); all
    // XMLSEC_VERSION_* variables in configure.ac are derived from it.
    content
        .lines()
        .find_map(|l| {
            let args = l.trim().strip_prefix("AC_INIT(")?;
            let version = args.split(',').nth(1)?;
            Some(
                version
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
            )
        })
        .expect("cannot find AC_INIT version in xmlsec/configure.ac")
}

/// Builder for the vendored xmlsec library.
#[derive(Default)]
pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    host: Option<String>,
    openssl_include_dirs: Vec<PathBuf>,
    libxml2_include_dirs: Vec<PathBuf>,
}

/// Result of a successful [`Build::build`].
pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    libs: Vec<String>,
    defines: Vec<(String, Option<String>)>,
    version: String,
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("xmlsec-build")),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
            ..Build::default()
        }
    }

    /// Directory the headers and library are installed into.
    /// Defaults to `$OUT_DIR/xmlsec-build`.
    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    /// Adds an OpenSSL include directory.
    ///
    /// If none is given, `DEP_OPENSSL_INCLUDE` (set by `openssl-sys` for crates
    /// depending on it) is used, falling back to the compiler's default search path.
    pub fn openssl_include_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.openssl_include_dirs.push(path.as_ref().to_path_buf());
        self
    }

    /// Adds a libxml2 include directory (the one containing `libxml/`).
    ///
    /// If none is given, libxml2 is located with vcpkg on MSVC targets and
    /// pkg-config everywhere else.
    pub fn libxml2_include_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.libxml2_include_dirs.push(path.as_ref().to_path_buf());
        self
    }

    /// Compiles xmlsec and installs headers and the static library into the output directory.
    ///
    /// Panics on failure, as is customary in build scripts.
    pub fn build(&mut self) -> Artifacts {
        let out_dir = self.out_dir.clone().expect("OUT_DIR not set");
        let target = self.target.clone().expect("TARGET not set");
        let host = self.host.clone().expect("HOST not set");

        let src_dir = source_dir();
        let include_dir = out_dir.join("include");
        let lib_dir = out_dir.join("lib");
        let version = version();

        install_headers(&src_dir, &include_dir, &version);
        fs::create_dir_all(&lib_dir).expect("cannot create lib dir");

        let mut cc = cc::Build::new();
        cc.target(&target)
            .host(&host)
            .out_dir(&lib_dir)
            .cargo_metadata(false)
            .warnings(false)
            .include(&include_dir)
            .includes(self.openssl_include_dirs(&target))
            .includes(self.libxml2_include_dirs(&target))
            .files(SOURCES.iter().map(|s| src_dir.join("src").join(s)));
        for (name, value) in DEFINES {
            cc.define(name, *value);
        }
        cc.compile(LIB_NAME);

        Artifacts {
            include_dir,
            lib_dir,
            libs: vec![LIB_NAME.to_string()],
            defines: DEFINES
                .iter()
                .map(|(n, v)| (n.to_string(), v.map(str::to_string)))
                .collect(),
            version,
        }
    }

    fn openssl_include_dirs(&self, _target: &str) -> Vec<PathBuf> {
        if !self.openssl_include_dirs.is_empty() {
            return self.openssl_include_dirs.clone();
        }
        env::var_os("DEP_OPENSSL_INCLUDE")
            .map(|p| env::split_paths(&p).collect())
            .unwrap_or_default()
    }

    fn libxml2_include_dirs(&self, target: &str) -> Vec<PathBuf> {
        if !self.libxml2_include_dirs.is_empty() {
            return self.libxml2_include_dirs.clone();
        }

        // Only the headers are needed here; linking libxml2 is up to the `libxml` crate.
        if target.contains("msvc") {
            let lib = vcpkg::Config::new()
                .cargo_metadata(false)
                .find_package("libxml2")
                .expect("libxml2 not found via vcpkg, use Build::libxml2_include_dir");
            lib.include_paths
                .into_iter()
                .flat_map(|p| [p.join("libxml2"), p])
                .collect()
        } else {
            let lib = pkg_config::Config::new()
                .cargo_metadata(false)
                .probe("libxml-2.0")
                .expect("libxml2 not found via pkg-config, use Build::libxml2_include_dir");
            lib.include_paths
        }
    }
}

impl Artifacts {
    /// Directory containing the `xmlsec/` headers, including the generated `version.h`.
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    /// Preprocessor defines required when compiling against the headers.
    pub fn defines(&self) -> &[(String, Option<String>)] {
        &self.defines
    }

    /// Version of the vendored xmlsec, e.g. `"1.3.13"`.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Emits the cargo instructions to link the static library and exposes
    /// `DEP_<links>_INCLUDE`/`DEP_<links>_LIB` to dependents of the calling crate.
    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in &self.libs {
            println!("cargo:rustc-link-lib=static={lib}");
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}

/// Copies the public headers and generates `xmlsec/version.h` from `version.h.in`.
fn install_headers(src_dir: &Path, include_dir: &Path, version: &str) {
    let src_include = src_dir.join("include").join("xmlsec");
    let dst_include = include_dir.join("xmlsec");

    for sub in ["", "openssl"] {
        let src = src_include.join(sub);
        let dst = dst_include.join(sub);
        fs::create_dir_all(&dst).expect("cannot create include dir");
        for entry in fs::read_dir(&src).expect("cannot read xmlsec include dir") {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "h") {
                fs::copy(&path, dst.join(path.file_name().unwrap())).expect("cannot copy header");
            }
        }
    }

    let parts: Vec<u32> = version
        .split('.')
        .map(|p| p.parse().expect("invalid xmlsec version number"))
        .collect();
    assert_eq!(parts.len(), 3, "unexpected xmlsec version format: {version}");
    let (major, minor, subminor) = (parts[0], parts[1], parts[2]);
    // Same formula as configure.ac: current = 10000*major + 100*minor + subminor
    let version_info = format!("{}:0:0", 10000 * major + 100 * minor + subminor);

    let header = fs::read_to_string(src_include.join("version.h.in"))
        .expect("cannot read version.h.in")
        .replace("@XMLSEC_VERSION@", version)
        .replace("@XMLSEC_VERSION_MAJOR@", &major.to_string())
        .replace("@XMLSEC_VERSION_MINOR@", &minor.to_string())
        .replace("@XMLSEC_VERSION_SUBMINOR@", &subminor.to_string())
        .replace("@XMLSEC_VERSION_INFO@", &version_info);
    fs::write(dst_include.join("version.h"), header).expect("cannot write version.h");
}
