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
 * References:
 *  https://docs.kernel.org/i2c/dev-interface.html
 *  https://www.kernel.org/doc/Documentation/i2c/dev-interface
 *  https://docs.kernel.org/driver-api/i2c.html
 *
 */

use crate::prelude::*;
use afbv4::prelude::*;
use std::cell::Cell;
use std::ffi::CStr;
use std::ffi::CString;
use std::str;

pub struct I2cHandle {
    devname: CString,
    raw_fd: Cell<i32>,
}

impl I2cHandle {
    pub fn new(i2cbus: &'static str) -> Result<I2cHandle, AfbError> {
        let devname = match CString::new(i2cbus) {
            Err(_) => {
                return Err(AfbError::new(
                    "serial-invalid-devname",
                    "fail to convert name to UTF8",
                ))
            }
            Ok(value) => value,
        };

        let handle = I2cHandle {
            devname: devname,
            raw_fd: Cell::new(0),
        };

        // open the line before returning the handle
        let _ = &handle.open()?;
        Ok(handle)
    }

    pub fn open(&self) -> Result<(), AfbError> {
        // open tty i2cbus
        let raw_fd = unsafe { cglue::open(self.devname.as_ptr(), cglue::BUS_I2C_O_RDWR, 0) };
        if raw_fd < 0 {
            return Err(AfbError::new("serial-open-fail", get_perror()));
        }

        // update fd cell within immutable handle
        self.raw_fd.set(raw_fd);
        afb_log_msg!(Debug, None, "Open port={:?}", self.devname);

        Ok(())
    }

    pub fn close(&self) {
        unsafe { cglue::close(self.raw_fd.get()) };
    }

    pub fn reopen(&self) -> Result<(), AfbError> {
        self.close();
        self.open()
    }

    pub fn get_fd(&self) -> i32 {
        self.raw_fd.get()
    }

    pub fn get_name<'a>(&'a self) -> &'a CStr {
        self.devname.as_c_str()
    }

    pub fn read<T>(&self, addr: u32, reg: u8) -> Result<T, AfbError>
    where
        I2cHandle: I2cDataCmd<T>,
    {
        let fd = self.raw_fd.get();

        if (unsafe { cglue::ioctl(fd, cglue::BUS_I2C_SLAVE, &addr) } < 0) {
            return Err(AfbError::new(
                "ic2-read-addr",
                format!("invalid addr={}", addr),
            ));
        }

        match I2cHandle::mk_read(fd, reg) {
            Err(error) => Err(AfbError::new(
                "ic2-read-data",
                format!("addr:{} register:{} error:{}", addr, reg, error),
            )),
            Ok(value) => Ok(value),
        }
    }

    pub fn write<T>(&self, addr: u32, reg: u8, data: T) -> Result<(), AfbError>
    where
        I2cHandle: I2cDataCmd<T>,
    {
        let fd = self.raw_fd.get();

        if (unsafe { cglue::ioctl(fd, cglue::BUS_I2C_SLAVE, &addr) } < 0) {
            return Err(AfbError::new(
                "ic2-write-addr",
                format!("invalid addr={}", addr),
            ));
        }

        match I2cHandle::mk_write(fd, reg, data) {
            Err(error) => Err(AfbError::new(
                "ic2-write-data",
                format!("addr:{} register:{} error:{}", addr, reg, error),
            )),
            Ok(value) => Ok(value),
        }
    }
}

impl I2cDataCmd<u8> for I2cHandle {
    fn mk_read(fd: i32, register: u8) -> Result<u8, String> {
        let res = unsafe { cglue::i2c_smbus_read_byte_data(fd, register) };
        if res < 0 {
            return Err(get_perror());
        }
        Ok((res & 0xFF) as u8)
    }

    fn mk_write(fd: i32, register: u8, data: u8) -> Result<(), String> {
        let res = unsafe { cglue::i2c_smbus_write_byte_data(fd, register, data) };
        if res < 0 {
            return Err(get_perror());
        }
        Ok(())
    }
}

impl I2cDataCmd<u16> for I2cHandle {
    fn mk_read(fd: i32, register: u8) -> Result<u16, String> {
        let res = unsafe { cglue::i2c_smbus_read_word_data(fd, register) };
        if res < 0 {
            return Err(get_perror());
        }
        Ok((res & 0xFFFF) as u16)
    }

    fn mk_write(fd: i32, register: u8, data: u16) -> Result<(), String> {
        let res = unsafe { cglue::i2c_smbus_write_word_data(fd, register, data) };
        if res < 0 {
            return Err(get_perror());
        }
        Ok(())
    }
}

pub trait I2cDataCmd<T> {
    fn mk_read(fd: i32, register: u8) -> Result<T, String>;
    fn mk_write(fd: i32, register: u8, data: T) -> Result<(), String>;
}
