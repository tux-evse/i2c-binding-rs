/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/
extern crate bindgen;

fn main() {
    // invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=capi/capi-map.h");

    let header = "
    // -----------------------------------------------------------------------
    //         <- private '_capi-map.rs' Rust/C unsafe binding ->
    // -----------------------------------------------------------------------
    //   Do not exit this file it will be regenerated automatically by cargo.
    //   Check:
    //     - build.rs for C/Rust glue options
    //     - src/capi/capi-map.h for C prototype inputs
    // -----------------------------------------------------------------------
    ";
    let _capi_map
     = bindgen::Builder::default()
        .header("capi/capi-map.h") // Pionix C++ prototype wrapper input
        .raw_line(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(false)
        .layout_tests(false)

        .allowlist_function("open")
        .allowlist_function("close")
        .allowlist_function("ioctl")
        .allowlist_function("i2c_smbus_read_.*")
        .allowlist_function("i2c_smbus_write_.*")

        .allowlist_var("BUS_I2C_.*")

        .allowlist_function("__errno_location")
        .allowlist_function("errno")
        .allowlist_function("strerror_r")

        .generate()
        .expect("Unable to generate _capi-map.rs");

    _capi_map
        .write_to_file("capi/_capi-map.rs")
        .expect("Couldn't write _capi-map.rs!");
}
