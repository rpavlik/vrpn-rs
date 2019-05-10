// Copyright 2018, Collabora, Ltd.
// SPDX-License-Identifier: BSL-1.0
// Author: Ryan A. Pavlik <ryan.pavlik@collabora.com>

use crate::async_io::codec::*;
use crate::async_io::cookie::*;
use crate::{ClassOfService, Endpoint, GenericMessage, Result, SystemMessage, TranslationTables};
use futures::sync::mpsc;
use std::fs;
use tokio::{
    codec::{Decoder, Framed},
    fs::File,
    prelude::*,
};

pub struct EndpointFile {
    translation: TranslationTables,
    file: Framed<File, FramedMessageCodec>,
    system_rx: mpsc::UnboundedReceiver<SystemMessage>,
    system_tx: mpsc::UnboundedSender<SystemMessage>,
}

impl EndpointFile {
    pub fn new(file: fs::File) -> Result<EndpointFile> {
        let (system_tx, system_rx) = mpsc::unbounded();
        let file = File::from_std(file);
        let file = read_and_check_file_cookie(file).wait()?;
        Ok(EndpointFile {
            translation: TranslationTables::new(),
            file: FramedMessageCodec.framed(file),
            system_tx,
            system_rx,
        })
    }
}
impl Endpoint for EndpointFile {
    fn translation_tables(&self) -> &TranslationTables {
        &self.translation
    }
    fn translation_tables_mut(&mut self) -> &mut TranslationTables {
        &mut self.translation
    }

    fn send_system_change(&self, _message: SystemMessage) -> Result<()> {
        unimplemented!()
    }

    fn buffer_generic_message(
        &mut self,
        _msg: GenericMessage,
        _class: ClassOfService,
    ) -> Result<()> {
        unimplemented!()
    }
}
