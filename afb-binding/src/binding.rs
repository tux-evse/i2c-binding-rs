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
 */

use crate::prelude::*;
use afbv4::prelude::*;

pub(crate) fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

pub(crate) struct BindingCfg {
    pub i2cbus: &'static str,
    pub devices: JsoncObj,
}

impl AfbApiControls for BindingCfg {
    fn config(&mut self, api: &AfbApi, jconf: JsoncObj) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api={} config={}", api.get_uid(), jconf);
        Ok(())
    }

    // mandatory for downcasting back to custom api data object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    let uid = if let Ok(value) = jconf.get::<String>("uid") {
        to_static_str(value)
    } else {
        "i2c"
    };

    let api = if let Ok(value) = jconf.get::<String>("api") {
        to_static_str(value)
    } else {
        uid
    };

    let info = if let Ok(value) = jconf.get::<String>("info") {
        to_static_str(value)
    } else {
        ""
    };

    let permission = if let Ok(value) = jconf.get::<String>("permission") {
        AfbPermission::new(to_static_str(value))
    } else {
        AfbPermission::new("acl:i2c:client")
    };

    let i2cbus = if let Ok(value) = jconf.get::<String>("i2cbus") {
        to_static_str(value)
    } else {
        return afb_error!(
            "i2c-config-fail",
            "mandatory label 'i2cbus' device missing",
        )
    };

    let devices = if let Ok(value) = jconf.get::<JsoncObj>("devices") {
        if !matches!(value.get_type(), Jtype::Array) {
            return afb_error!(
                "i2c-config-fail",
                "mandatory 'devices' should be an array",
            )
        }
        value
    } else {
        return afb_error!(
            "i2c-config-fail",
            "mandatory 'devices' config missing",
        );
    };

    let config = BindingCfg {
        i2cbus,
        devices,
    };

    // create backend API
    let api = AfbApi::new(api).set_info(info).set_permission(permission);
    register_verbs(api, config)?;

    Ok(api.finalize()?)
}

// register binding within libafb
AfbBindingRegister!(binding_init);
