// Copyright 2018, Collabora, Ltd.
// SPDX-License-Identifier: BSL-1.0
// Author: Ryan A. Pavlik <ryan.pavlik@collabora.com>

use crate::{ping::Client as RawClient, Connection, Error, LocalId, Result, SenderId, SenderName};
use futures::{ready, Stream};
use std::task::Poll;
use std::{sync::Arc, time::Duration};
use tokio::prelude::*;
use tokio::time::Interval;

pub struct Client<T: Connection + 'static> {
    client: RawClient<T>,
    interval: Interval,
}

impl<T: Connection + 'static> Client<T> {
    fn new_impl(client: RawClient<T>) -> Result<Client<T>> {
        Ok(Client {
            client,
            interval: Interval::new_interval(Duration::from_secs(1)),
        })
    }
    pub fn new(sender: LocalId<SenderId>, connection: Arc<T>) -> Result<Client<T>> {
        Client::new_impl(RawClient::new(sender, connection)?)
    }

    pub fn new_from_name(
        sender: impl Into<SenderName> + Clone,
        connection: Arc<T>,
    ) -> Result<Client<T>> {
        Client::new_impl(RawClient::new_from_name(sender, connection)?)
    }
}

impl<T: Connection + 'static> Stream for Client<T> {
    type Item = Result<(), Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let _ = ready!(self
            .interval
            .poll()
            .map_err(|e| Error::OtherMessage(e.to_string())));

        // match self.interval.poll()? {
        //     Async::NotReady => Ok(Async::NotReady),
        //     Async::Ready(Some(_)) => {
        if let Some(radio_silence) = self.client.check_ping_cycle()? {
            eprintln!(
                "It has been {} since the first unanwered ping was sent to the server!",
                radio_silence
            );
        }
        Ok(Poll::Ready(Some(())))
        //     }
        //     Async::Ready(None) => Ok(Async::Ready(None)),
        // }
    }
}
