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
use libi2c::prelude::*;
use std::rc::Rc;
use std::time::Duration;
use std::{thread, time};

fn hexa_string_to_u32(input: String) -> Result<u32, AfbError> {
    let data = input.trim_start_matches("0x");
    if data != input {
        match u32::from_str_radix(data, 16) {
            Err(_error) => afb_error!("hexa-invalid-u32", input),
            Ok(value) => Ok(value),
        }
    } else {
        match u32::from_str_radix(data, 10) {
            Err(_error) => afb_error!("hexa-invalid-integer", input),
            Ok(value) => Ok(value),
        }
    }
}

fn hexa_string_to_u16(input: String) -> Result<u16, AfbError> {
    let data = input.trim_start_matches("0x");
    if data != input {
        match u16::from_str_radix(data, 16) {
            Err(_error) => afb_error!("hexa-invalid-u16", input),
            Ok(value) => Ok(value),
        }
    } else {
        match u16::from_str_radix(data, 10) {
            Err(_error) => afb_error!("hexa-invalid-integer", input),
            Ok(value) => Ok(value),
        }
    }
}

fn hexa_string_to_u8(input: String) -> Result<u8, AfbError> {
    let data = input.trim_start_matches("0x");
    if data != input {
        match u8::from_str_radix(data, 16) {
            Err(_error) => afb_error!("hexa-invalid-u8", input),
            Ok(value) => Ok(value),
        }
    } else {
        match u8::from_str_radix(data, 10) {
            Err(_error) => afb_error!("hexa-invalid-integer", input),
            Ok(value) => Ok(value as u8),
        }
    }
}

pub(self) fn cmd_exec(
    i2c: Rc<I2cHandle>,
    dev_addr: u32,
    cmd_size: u8,
    cmd: JsoncObj,
) -> Result<(), AfbError> {
    let cmd_reg = hexa_string_to_u8(cmd.get::<String>("reg")?)?;
    let cmd_value = cmd.get::<String>("value")?;
    match cmd_size {
        1 => {
            i2c.write(dev_addr, cmd_reg, hexa_string_to_u8(cmd_value)?)?;
        }
        2 => {
            i2c.write(dev_addr, cmd_reg, hexa_string_to_u16(cmd_value)?)?;
        }
        _ => {
            return afb_error!(
                "i2c-init-size",
                "invalid size:{} should Byte(1) & World(2)", cmd_size
            )
        }
    }
    Ok(())
}

#[derive(Clone)]
struct PresetData {
    delay: Option<Duration>,
    values: Vec<u16>,
}

#[derive(Clone)]
enum PresetValue {
    READ,
    WRITE,
    PRESET(PresetData),
}

#[derive(Clone)]
struct PreSetAction {
    action: String,
    value: PresetValue,
}

struct RqtI2ccCtx {
    i2c: Rc<I2cHandle>,
    actions: Vec<PreSetAction>,
    dev_addr: u32,
    cmd_reg: u8,
    cmd_size: u8,
}

struct RqtI2ccDataCtx {
    i2c: Rc<I2cHandle>,
    actions: Vec<PreSetAction>,
    dev_addr: u32,
    cmd_reg: u8,
    cmd_size: u8,
}

fn rqt_i2c_cb(rqt: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {

    let ctx = ctx.get_ref::<RqtI2ccDataCtx>()?;
    let query = args.get::<JsoncObj>(0)?;
    let action = query.get::<String>("action")?.to_lowercase();

    for preset in &ctx.actions {
        if action == preset.action {
            match &preset.value {
                PresetValue::READ => match ctx.cmd_size {
                    1 => {
                        let data: u8 = ctx.i2c.read(ctx.dev_addr, ctx.cmd_reg)?;
                        rqt.reply(data as u32, 0);
                    }
                    2 => {
                        let data: u16 = ctx.i2c.read(ctx.dev_addr, ctx.cmd_reg)?;
                        rqt.reply(data as u32, 0);
                    }
                    _ => {
                        return afb_error!(
                            "rqt-i2c-size",
                           "invalid size:{} should Byte(1) & World(2)", ctx.cmd_size
                        )
                    }
                },
                PresetValue::WRITE => {
                    let query = query.get::<String>("value")?;
                    match ctx.cmd_size {
                        1 => {
                            ctx.i2c
                                .write(ctx.dev_addr, ctx.cmd_reg, hexa_string_to_u8(query)?)?;
                            rqt.reply(AFB_NO_DATA, 0);
                        }
                        2 => {
                            ctx.i2c
                                .write(ctx.dev_addr, ctx.cmd_reg, hexa_string_to_u16(query)?)?;
                            rqt.reply(AFB_NO_DATA, 0);
                        }
                        _ => {
                            return afb_error!(
                                "rqt-i2c-size",
                                "invalid size:{} should Byte(1) & World(2)", ctx.cmd_size
                            )
                        }
                    }
                }
                // loop on preset value if needed wait except for last preset
                PresetValue::PRESET(data) => match ctx.cmd_size {
                    1 => {
                        let count = data.values.len();
                        for idx in 0..count {
                            ctx.i2c
                                .write(ctx.dev_addr, ctx.cmd_reg, data.values[idx] as u8)?;
                            if let Some(value) = data.delay {
                                if idx < count - 1 {
                                    thread::sleep(value)
                                }
                            }
                        }
                        rqt.reply(AFB_NO_DATA, 0);
                    }
                    2 => {
                        let count = data.values.len();
                        for idx in 0..count {
                            ctx.i2c
                                .write(ctx.dev_addr, ctx.cmd_reg, data.values[idx] as u16)?;
                            if let Some(value) = data.delay {
                                if idx < count - 1 {
                                    thread::sleep(value)
                                }
                            }
                        }
                        rqt.reply(AFB_NO_DATA, 0);
                    }
                    _ => {
                        return afb_error!(
                            "rqt-i2c-size",
                            "invalid size:{} should Byte(1) & World(2)", ctx.cmd_size
                        )
                    }
                },
            }
        }
    }
    Ok(())
}

pub(crate) fn register_verbs(api: &mut AfbApi, config: BindingCfg) -> Result<(), AfbError> {
    // default actions
    let get = PreSetAction {
        action: "set".to_string(),
        value: PresetValue::WRITE,
    };
    let set = PreSetAction {
        action: "get".to_string(),
        value: PresetValue::READ,
    };

    // open i2c bus and send init commands if needed
    let i2c = Rc::new(I2cHandle::new(config.i2cbus)?);

    // loop on command and create corresponding verbs
    for idx in 0..config.devices.count()? {
        let device = config.devices.index::<JsoncObj>(idx)?;

        let dev_uid = to_static_str(device.get::<String>("uid")?);
        let group = AfbGroup::new(dev_uid);

        let group = match device.get::<String>("info") {
            Ok(value) => group.set_info(to_static_str(value)),
            Err(_) => group,
        };

        let group = match device.get::<String>("prefix") {
            Ok(value) => group.set_prefix(to_static_str(value)),
            Err(_) => group,
        };

        let group = match device.get::<String>("permission") {
            Ok(value) => group.set_permission(AfbPermission::new(to_static_str(value))),
            Err(_) => group,
        };

        // mandatory I2C device fields
        let dev_addr = hexa_string_to_u32(device.get::<String>("addr")?)?;
        let dev_size = if let Ok(value) = device.get::<u32>("size") {
            value as u8
        } else {
            1
        };

        let dev_delay = if let Ok(value) = device.get::<u64>("delay") {
            Some(time::Duration::from_millis(value))
        } else {
            None
        };

        // check device need to be initialized
        if let Ok(inits) = device.get::<JsoncObj>("init") {
            match inits.get_type() {
                Jtype::Array => {
                    for kdx in 0..inits.count()? {
                        let init = inits.index::<JsoncObj>(kdx)?;
                        cmd_exec(i2c.clone(), dev_addr, dev_size, init.clone())?;
                    }
                }
                _ => {
                    return afb_error!(
                        "i2c-config-fail",
                                                    "device:{} optional 'init' label should be an array",
                            dev_uid

                    )
                }
            }
        }

        let cmds = if let Ok(value) = device.get::<JsoncObj>("cmds") {
            if !matches!(value.get_type(), Jtype::Array) {
                return afb_error!(
                    "i2c-config-fail",
                    "device:{} 'cmds' should be an array", dev_uid
                )
            }
            value
        } else {
            return afb_error!(
                "i2c-config-fail",
                "device:{} 'cmds' config missing", dev_uid
            )
        };

        for jdx in 0..cmds.count()? {
            let cmd = cmds.index::<JsoncObj>(jdx)?;

            let cmd_uid = to_static_str(cmd.get::<String>("uid")?);
            let verb = AfbVerb::new(cmd_uid);

            let cmd_reg = hexa_string_to_u8(cmd.get::<String>("register")?)?;

            if let Ok(value) = cmd.get::<String>("info") {
                verb.set_info(to_static_str(value));
            };

            let cmd_size = if let Ok(value) = cmd.get::<u32>("size") {
                value as u8
            } else {
                dev_size
            };

            let cmd_delay = if let Ok(value) = cmd.get::<u64>("delay") {
                Some(time::Duration::from_millis(value))
            } else {
                dev_delay
            };

            if let Ok(value) = cmd.get::<String>("permission") {
                verb.set_permission(AfbPermission::new(to_static_str(value)));
            };

            // provision default actions and then config presets
            let mut actions = Vec::from([set.clone(), get.clone()]);
            let mut actions_info = "['get',".to_string();
            if let Ok(presets) = cmd.get::<JsoncObj>("presets") {
                for jdx in 0..presets.count()? {
                    let preset = presets.index::<JsoncObj>(jdx)?;
                    let action = preset.get::<String>("action")?.to_lowercase();
                    let mut data = PresetData {
                        delay: cmd_delay,
                        values: Vec::new(),
                    };
                    let values = preset.get::<JsoncObj>("values")?;
                    for kdx in 0..values.count()? {
                        let value = hexa_string_to_u16(values.index::<String>(kdx)?)?;
                        data.values.push(value);
                    }
                    actions_info.push_str(format!("'{}',", &action).as_str());
                    actions.push(PreSetAction {
                        action: action,
                        value: PresetValue::PRESET(data),
                    });
                }
            } else {
                actions_info.push_str("'set'");
                verb.set_usage("{'action':'set|get', 'value':'0x??'");
                if let Ok(samples) = cmd.get::<JsoncObj>("samples") {
                    for kdx in 0..samples.count()? {
                        let sample = samples.index::<String>(kdx)?;
                        verb.add_sample(to_static_str(format!(
                            "{{'action':'set','value':'{}'}}",
                            sample
                        )))?;
                    }
                }
            };
            actions_info.push_str("]"); // close action info json_string array
            verb.set_actions(to_static_str(actions_info))?;

            verb.set_callback(rqt_i2c_cb);
            verb.set_context(RqtI2ccDataCtx {
                i2c: i2c.clone(),
                actions,
                dev_addr,
                cmd_reg,
                cmd_size: cmd_size,
            });

            // add command to current group
            let group = unsafe { &mut *(group as *mut AfbGroup) };
            group.add_verb(verb.finalize()?);
        }

        // add command group to api
        let group = unsafe { &mut *(group as *mut AfbGroup) };
        api.add_group(group.finalize()?);
    }
    Ok(())
}
