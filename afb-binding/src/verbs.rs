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
            Err(_error) => Err(AfbError::new("hexa-invalid-u32", input)),
            Ok(value) => Ok(value),
        }
    } else {
        match u32::from_str_radix(data, 10) {
            Err(_error) => Err(AfbError::new("hexa-invalid-integer", input)),
            Ok(value) => Ok(value),
        }
    }
}

fn hexa_string_to_u16(input: String) -> Result<u16, AfbError> {
    let data = input.trim_start_matches("0x");
    if data != input {
        match u16::from_str_radix(data, 16) {
            Err(_error) => Err(AfbError::new("hexa-invalid-u16", input)),
            Ok(value) => Ok(value),
        }
    } else {
        match u16::from_str_radix(data, 10) {
            Err(_error) => Err(AfbError::new("hexa-invalid-integer", input)),
            Ok(value) => Ok(value),
        }
    }
}

fn hexa_string_to_u8(input: String) -> Result<u8, AfbError> {
    let data = input.trim_start_matches("0x");
    if data != input {
        match u8::from_str_radix(data, 16) {
            Err(_error) => Err(AfbError::new("hexa-invalid-u8", input)),
            Ok(value) => Ok(value),
        }
    } else {
        match u8::from_str_radix(data, 10) {
            Err(_error) => Err(AfbError::new("hexa-invalid-integer", input)),
            Ok(value) => Ok(value as u8),
        }
    }
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
    register: u8,
    addr: u32,
    size: u8,
}

AfbVerbRegister!(RqtI2ccVerb, rqt_i2c_cb, RqtI2ccCtx);
fn rqt_i2c_cb(rqt: &AfbRequest, args: &AfbData, ctx: &mut RqtI2ccCtx) -> Result<(), AfbError> {
    let query = args.get::<JsoncObj>(0)?;
    let action = query.get::<String>("action")?.to_lowercase();

    for preset in &ctx.actions {
        if action == preset.action {
            match &preset.value {
                PresetValue::READ => match ctx.size {
                    1 => {
                        let data: u8 = ctx.i2c.read(ctx.addr, ctx.register)?;
                        rqt.reply(data as u32, 0);
                    }
                    2 => {
                        let data: u16 = ctx.i2c.read(ctx.addr, ctx.register)?;
                        rqt.reply(data as u32, 0);
                    }
                    _ => {
                        return Err(AfbError::new(
                            "rqt-i2c-size",
                            format!("invalid size:{} should Byte(1) & World(2)", ctx.size),
                        ))
                    }
                },
                PresetValue::WRITE => {
                    let query = query.get::<String>("value")?;
                    match ctx.size {
                        1 => {
                            ctx.i2c
                                .write(ctx.addr, ctx.register, hexa_string_to_u8(query)?)?;
                            rqt.reply(AFB_NO_DATA, 0);
                        }
                        2 => {
                            ctx.i2c
                                .write(ctx.addr, ctx.register, hexa_string_to_u16(query)?)?;
                            rqt.reply(AFB_NO_DATA, 0);
                        }
                        _ => {
                            return Err(AfbError::new(
                                "rqt-i2c-size",
                                format!("invalid size:{} should Byte(1) & World(2)", ctx.size),
                            ))
                        }
                    }
                }
                PresetValue::PRESET(data) => {
                    match ctx.size {
                        1 => {
                            for idx in 0..data.values.len() {
                                ctx.i2c
                                    .write(ctx.addr, ctx.register, data.values[idx] as u8)?;
                                if let Some(value) = data.delay {
                                    thread::sleep(value)
                                }
                            }
                            rqt.reply(AFB_NO_DATA, 0);
                        }
                        2 => {
                            for idx in 0..data.values.len() {
                                ctx.i2c
                                    .write(ctx.addr, ctx.register, data.values[idx] as u16)?;
                                if let Some(value) = data.delay {
                                    thread::sleep(value)
                                }
                            }
                            rqt.reply(AFB_NO_DATA, 0);
                        }
                        _ => {
                            return Err(AfbError::new(
                                "rqt-i2c-size",
                                format!("invalid size:{} should Byte(1) & World(2)", ctx.size),
                            ))
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn register_verbs(api: &mut AfbApi, config: BindingCfg) -> Result<(), AfbError> {
    // open i2c bus and send init commands if needed
    let i2c = Rc::new(I2cHandle::new(config.i2cbus)?);
    if let Some(inits) = config.inits {
        for idx in 0..inits.count()? {
            let cmd = inits.index::<JsoncObj>(idx)?;
            let addr = hexa_string_to_u32(cmd.get::<String>("addr")?)?;
            let register = hexa_string_to_u8(cmd.get::<String>("reg")?)?;
            let value = cmd.get::<String>("value")?;
            let size = if let Ok(value) = cmd.get::<u32>("size") {
                value
            } else {
                1
            };

            match size {
                1 => {
                    i2c.write(addr, register, hexa_string_to_u8(value)?)?;
                }
                2 => {
                    i2c.write(addr, register, hexa_string_to_u16(value)?)?;
                }
                _ => {
                    return Err(AfbError::new(
                        "i2c-init-size",
                        format!("invalid size:{} should Byte(1) & World(2)", size),
                    ))
                }
            }
        }
    }

    // default actions
    let get = PreSetAction {
        action: "set".to_string(),
        value: PresetValue::WRITE,
    };
    let set = PreSetAction {
        action: "get".to_string(),
        value: PresetValue::READ,
    };

    // loop on command and create corresponding verbs
    for idx in 0..config.cmds.count()? {
        let cmd = config.cmds.index::<JsoncObj>(idx)?;

        // mandatory fields
        let uid = cmd.get::<String>("uid")?;
        let addr = hexa_string_to_u32(cmd.get::<String>("addr")?)?;
        let register = hexa_string_to_u8(cmd.get::<String>("register")?)?;

        let name = if let Ok(value) = cmd.get::<String>("name") {
            Some(value)
        } else {
            None
        };

        let info = if let Ok(value) = cmd.get::<String>("info") {
            Some(value)
        } else {
            None
        };

        let permission = if let Ok(value) = cmd.get::<String>("info") {
            Some(value)
        } else {
            None
        };

        let size = if let Ok(value) = cmd.get::<u32>("size") {
            value as u8
        } else {
            1
        };

        let verb = AfbVerb::new(to_static_str(uid));

        // provision default actions and then config presets
        let mut actions = Vec::from([set.clone(), get.clone()]);
        let mut actions_info = "['get',".to_string();
        if let Ok(presets) = cmd.get::<JsoncObj>("presets") {
            for jdx in 0..presets.count()? {
                let preset = presets.index::<JsoncObj>(jdx)?;
                let action = preset.get::<String>("action")?.to_lowercase();
                let delay = if let Ok(value) = preset.get::<u64>("delay") {
                    Some(time::Duration::from_millis(value))
                } else {
                    None
                };
                let mut data = PresetData {
                    delay: delay,
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
                    verb.set_sample(to_static_str(format!(
                        "{{'action':'set','value':'{}'}}",
                        sample
                    )))?;
                }
            }
        };
        actions_info.push_str("]"); // close action info json_string array
        verb.set_action(to_static_str(actions_info))?;

        verb.set_callback(Box::new(RqtI2ccVerb {
            i2c: i2c.clone(),
            addr,
            register,
            actions,
            size: size,
        }));

        if let Some(value) = name {
            verb.set_name(to_static_str(value));
        }

        if let Some(value) = info {
            verb.set_info(to_static_str(value));
        }

        if let Some(value) = permission {
            verb.set_permission(AfbPermission::new(to_static_str(value)));
        }

        api.add_verb(verb.finalize()?);
    }

    Ok(())
}
