// Copyright 2018, Collabora, Ltd.
// SPDX-License-Identifier: BSL-1.0
// Author: Ryan A. Pavlik <ryan.pavlik@collabora.com>

use connection;
use connection::Endpoint;
use connection::LogFileNames;

use endpoint_ip::EndpointIP;
use typedispatcher::{HandlerResult, MappingResult, TypeDispatcher};
use types::{HandlerParams, SenderId, SenderName, TypeId, TypeName};

pub struct ConnectionIP {
    dispatcher: TypeDispatcher<'static>,
    remote_log_names: LogFileNames,
    local_log_names: LogFileNames,
    endpoints: Vec<Option<EndpointIP>>,
}

impl ConnectionIP {
    /// Common initialization
    fn init(&mut self) -> HandlerResult<()> {
        let handle_udp_message = |params: HandlerParams| -> HandlerResult<()> { Ok(()) };
        /*
        self.dispatcher
            .set_system_handler(constants::UDP_DESCRIPTION, handle_udp_message)
            */
        Ok(())
    }

    /// Create a new ConnectionIP that is a server.
    pub fn new_server(local_log_names: Option<LogFileNames>) -> HandlerResult<ConnectionIP> {
        let disp = TypeDispatcher::new()?;
        let mut ret = ConnectionIP {
            dispatcher: disp,
            remote_log_names: connection::make_none_log_names(),
            local_log_names: connection::make_log_names(local_log_names),
            endpoints: Vec::new(),
        };
        ret.init()?;
        Ok(ret)
    }

    /// Create a new ConnectionIP that is a client.
    pub fn new_client(
        local_log_names: Option<LogFileNames>,
        remote_log_names: Option<LogFileNames>,
    ) -> HandlerResult<ConnectionIP> {
        let disp = TypeDispatcher::new()?;
        let mut ret = ConnectionIP {
            dispatcher: disp,
            remote_log_names: connection::make_log_names(remote_log_names),
            local_log_names: connection::make_log_names(local_log_names),
            endpoints: Vec::new(),
        };
        // Create our single endpoint
        ret.endpoints.push(Some(EndpointIP::new()));
        ret.init()?;
        Ok(ret)
    }
}

impl<'a> connection::Connection<'a> for ConnectionIP {
    type EndpointItem = EndpointIP;
    type EndpointIteratorMut = std::slice::IterMut<'a, Option<EndpointIP>>;
    type EndpointIterator = std::slice::Iter<'a, Option<EndpointIP>>;

    fn endpoints_iter_mut(&'a mut self) -> Self::EndpointIteratorMut {
        self.endpoints.iter_mut()
    }

    fn endpoints_iter(&'a self) -> Self::EndpointIterator {
        self.endpoints.iter()
    }

    fn add_type(&mut self, name: TypeName) -> MappingResult<TypeId> {
        self.dispatcher.add_type(name)
    }

    fn add_sender(&mut self, name: SenderName) -> MappingResult<SenderId> {
        self.dispatcher.add_sender(name)
    }
    /// Returns the ID for the type name, if found.
    fn get_type_id(&self, name: &TypeName) -> Option<TypeId> {
        self.dispatcher.get_type_id(name)
    }
    /// Returns the ID for the sender name, if found.
    fn get_sender_id(&self, name: &SenderName) -> Option<SenderId> {
        self.dispatcher.get_sender_id(name)
    }
}
