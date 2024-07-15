/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIFNS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitaTIFns under the License.
 */

use ::std::os::raw;
use std::ffi::CStr;

const MAX_ERROR_LEN: usize = 256;
pub mod cglue {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!("_capi-map.rs");
}

pub fn get_p_error() -> String {
    let mut buffer = [0 as raw::c_char; MAX_ERROR_LEN];
    unsafe {
        cglue::strerror_r(
            *cglue::__errno_location(),
            &mut buffer as *mut raw::c_char,
            MAX_ERROR_LEN,
        )
    };
    let cstring = unsafe { CStr::from_ptr(&mut buffer as *const raw::c_char) };
    let slice: &str = cstring.to_str().unwrap();
    slice.to_owned()
}
