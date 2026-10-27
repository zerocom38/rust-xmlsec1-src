// Make sure libxml2 and OpenSSL get linked.
extern crate libxml;
extern crate openssl_sys;

use std::os::raw::c_int;

unsafe extern "C" {
    pub fn xmlSecInit() -> c_int;
    pub fn xmlSecShutdown() -> c_int;
    pub fn xmlSecOpenSSLInit() -> c_int;
    pub fn xmlSecOpenSSLShutdown() -> c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_shutdown() {
        unsafe {
            assert_eq!(xmlSecInit(), 0);
            assert_eq!(xmlSecOpenSSLInit(), 0);
            assert_eq!(xmlSecOpenSSLShutdown(), 0);
            assert_eq!(xmlSecShutdown(), 0);
        }
    }
}
