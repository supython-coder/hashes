#![no_std]

use digest::dev::digest_test;
use digest::new_test;

new_test!(
    streebog256_main,
    "streebog256",
    streebog::Streebog256,
    digest_test
);
new_test!(
    streebog512_main,
    "streebog512",
    streebog::Streebog512,
    digest_test
);
