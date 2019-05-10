// Copyright 2018, Collabora, Ltd.
// SPDX-License-Identifier: BSL-1.0
// Author: Ryan A. Pavlik <ryan.pavlik@collabora.com>

use bytes::{BufMut, Bytes, BytesMut};
use crate::prelude::*;
use crate::{
    constants::ALIGN, Buffer, BufferSize, BytesRequired, EmptyResult, Error, IdType, IntoId,
    Result, SenderId, SequenceNumber, StaticTypeName, TimeVal, TypeId, TypeSafeId, Unbuffer,
};
use std::mem::size_of;

/// Empty trait used to indicate types that can be placed in a message body.
pub trait MessageBody /*: Buffer + Unbuffer */ {}

/// The identification used for a typed message body type.
#[derive(Debug)]
pub enum MessageTypeIdentifier {
    /// User message types are identified by a string which is dynamically associated
    /// with an ID on each side.
    UserMessageName(StaticTypeName),

    /// System message types are identified by a constant, negative message type ID.
    ///
    /// TODO: find a way to assert/enforce that this is negative - maybe a SystemTypeId type?
    SystemMessageId(TypeId),
}

/// Trait for typed message bodies.
///
pub trait TypedMessageBody: std::fmt::Debug {
    /// The name string (for user messages) or type ID (for system messages) used to identify this message type.
    const MESSAGE_IDENTIFIER: MessageTypeIdentifier;
}

impl<T> MessageBody for T where T: TypedMessageBody /*+ Buffer + Unbuffer*/ {}

/// Header information for a message.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageHeader {
    pub time: TimeVal,
    pub message_type: TypeId,
    pub sender: SenderId,
}

impl MessageHeader {
    pub fn new(
        time: Option<TimeVal>,
        message_type: impl IntoId<BaseId = TypeId>,
        sender: impl IntoId<BaseId = SenderId>,
    ) -> MessageHeader {
        MessageHeader {
            time: time.unwrap_or_else(|| TimeVal::get_time_of_day()),
            message_type: message_type.into_id(),
            sender: sender.into_id(),
        }
    }
}

/// A message with header information, almost ready to be buffered to the wire.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Message<T: MessageBody> {
    pub header: MessageHeader,
    pub body: T,
}

pub type GenericMessage = Message<GenericBody>;

impl<T: MessageBody> Message<T> {
    pub fn new(
        time: Option<TimeVal>,
        message_type: impl IntoId<BaseId = TypeId>,
        sender: impl IntoId<BaseId = SenderId>,
        body: T,
    ) -> Message<T> {
        Message {
            header: MessageHeader::new(time, message_type, sender),
            body,
        }
    }

    /// Create a message by combining a header and a body.
    pub fn from_header_and_body(header: MessageHeader, body: T) -> Message<T> {
        Message { header, body }
    }

    /// Consumes this message and returns a new SequencedMessage, which the supplied sequence number has been added to.
    pub fn into_sequenced_message(self, sequence_number: SequenceNumber) -> SequencedMessage<T> {
        SequencedMessage {
            message: self,
            sequence_number,
        }
    }

    /// true if the message type indicates that it is a "system" message (type ID < 0)
    pub fn is_system_message(&self) -> bool {
        self.header.message_type.is_system_message()
    }
}

impl<T: TypedMessageBody + Unbuffer> Message<T> {
    pub fn try_from_generic(msg: &GenericMessage) -> Result<Message<T>> {
        let mut buf = msg.body.inner.clone();
        let body = T::unbuffer_ref(&mut buf)
            .map_need_more_err_to_generic_parse_err("parsing message body")?;
        if buf.len() > 0 {
            return Err(Error::OtherMessage(format!(
                "message body length was indicated as {}, but {} bytes remain unconsumed",
                msg.body.inner.len(),
                buf.len()
            )));
        }
        Ok(Message::from_header_and_body(msg.header.clone(), body))
    }
}

impl<T: TypedMessageBody + Buffer> Message<T> {
    pub fn try_into_generic(self) -> Result<GenericMessage> {
        let old_body = self.body;
        let header = self.header;
        BytesMut::new()
            .allocate_and_buffer(old_body)
            .and_then(|body| {
                Ok(GenericMessage::from_header_and_body(
                    header,
                    GenericBody::new(body.freeze()),
                ))
            })
    }
}

/// A message with header information and sequence number, ready to be buffered to the wire.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SequencedMessage<T: MessageBody> {
    pub message: Message<T>,
    pub sequence_number: SequenceNumber,
}

pub type SequencedGenericMessage = SequencedMessage<GenericBody>;

impl<T: MessageBody> SequencedMessage<T> {
    pub fn new(
        time: Option<TimeVal>,
        message_type: TypeId,
        sender: SenderId,
        body: T,
        sequence_number: SequenceNumber,
    ) -> SequencedMessage<T> {
        SequencedMessage {
            message: Message::new(time, message_type, sender, body),
            sequence_number,
        }
    }
}

impl<T: MessageBody> From<SequencedMessage<T>> for Message<T> {
    fn from(v: SequencedMessage<T>) -> Message<T> {
        v.message
    }
}

/// Generic body struct used in unbuffering process, before dispatch on type to fully decode.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GenericBody {
    pub inner: Bytes,
}

impl GenericBody {
    pub fn new(inner: Bytes) -> GenericBody {
        GenericBody { inner }
    }
}

impl Default for GenericBody {
    fn default() -> GenericBody {
        GenericBody::new(Bytes::default())
    }
}

impl MessageBody for GenericBody {}

impl WrappedConstantSize for SequenceNumber {
    type WrappedType = u32;
    fn get(&self) -> Self::WrappedType {
        self.0
    }
    fn new(v: Self::WrappedType) -> Self {
        SequenceNumber(v)
    }
}

impl WrappedConstantSize for SenderId {
    type WrappedType = IdType;
    fn get(&self) -> Self::WrappedType {
        TypeSafeId::get(self)
    }
    fn new(v: Self::WrappedType) -> Self {
        SenderId(v)
    }
}

impl WrappedConstantSize for TypeId {
    type WrappedType = IdType;
    fn get(&self) -> Self::WrappedType {
        TypeSafeId::get(self)
    }
    fn new(v: Self::WrappedType) -> Self {
        TypeId(v)
    }
}

#[inline]
fn compute_padding(len: usize) -> usize {
    let remainder = len % ALIGN;
    if remainder != 0 {
        ALIGN - remainder
    } else {
        0
    }
}

#[inline]
fn padded(len: usize) -> usize {
    len + compute_padding(len)
}

/// Simple struct for wrapping all calculations related to Message<T> size.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageSize {
    // The unpadded size of a message body only
    pub unpadded_body_size: usize,
}

pub type LengthField = u32;

impl MessageSize {
    const UNPADDED_HEADER_SIZE: usize = 5 * 4;

    /// Get a MessageSize from the unpadded size of a message body only.
    #[inline]
    pub fn from_unpadded_body_size(unpadded_body_size: usize) -> MessageSize {
        MessageSize { unpadded_body_size }
    }

    /// Get a MessageSize from the total unpadded size of a message (header plus body)
    #[inline]
    pub fn from_unpadded_message_size(unpadded_message_size: usize) -> MessageSize {
        MessageSize::from_unpadded_body_size(
            unpadded_message_size - MessageSize::UNPADDED_HEADER_SIZE,
        )
    }
    /// Get a MessageSize from the length field of a message (padded header plus unpadded body)
    #[inline]
    pub fn from_length_field(length_field: LengthField) -> MessageSize {
        MessageSize::from_unpadded_body_size(
            length_field as usize - padded(MessageSize::UNPADDED_HEADER_SIZE),
        )
    }

    /// The unpadded size of just the message body.
    #[inline]
    pub fn unpadded_body_size(&self) -> usize {
        self.unpadded_body_size
    }

    /// The padded header size plus the unpadded body size.
    ///
    /// This is the value put in the message header's length field.
    #[inline]
    pub fn length_field(&self) -> LengthField {
        (self.unpadded_body_size + padded(MessageSize::UNPADDED_HEADER_SIZE)) as LengthField
    }

    /// The size of the body plus padding (multiple of ALIGN)
    #[inline]
    pub fn padded_body_size(&self) -> usize {
        padded(self.unpadded_body_size)
    }

    /// The number of padding bytes required to follow the message body.
    #[inline]
    pub fn body_padding(&self) -> usize {
        compute_padding(self.unpadded_body_size)
    }

    /// The total padded size of a message (header plus body, padding applied individually).
    ///
    /// This is the size of buffer actually required for this message.
    #[inline]
    pub fn padded_message_size(&self) -> usize {
        self.padded_body_size() + padded(MessageSize::UNPADDED_HEADER_SIZE)
    }
}

// Header is 5 i32s (padded to vrpn_ALIGN):
// - padded header size + unpadded body size
// - time stamp
// - sender
// - type
//
// The four bytes of "padding" are actually the sequence number,
// which are not "officially" part of the header.
//
// body is padded out to vrpn_ALIGN

fn generic_message_size(msg: &SequencedGenericMessage) -> MessageSize {
    MessageSize::from_unpadded_body_size(msg.message.body.inner.len())
}
impl BufferSize for SequencedMessage<GenericBody> {
    fn buffer_size(&self) -> usize {
        generic_message_size(self).padded_message_size()
    }
}

impl Buffer for SequencedMessage<GenericBody> {
    /// Serialize to a buffer.
    fn buffer_ref<T: BufMut>(&self, buf: &mut T) -> EmptyResult {
        let size = generic_message_size(self);
        if buf.remaining_mut() < size.padded_message_size() {
            return Err(Error::OutOfBuffer);
        }
        let length_field = size.length_field() as u32;

        Buffer::buffer_ref(&length_field, buf)
            .and_then(|()| self.message.header.time.buffer_ref(buf))
            .and_then(|()| self.message.header.sender.buffer_ref(buf))
            .and_then(|()| self.message.header.message_type.buffer_ref(buf))
            .and_then(|()| self.sequence_number.buffer_ref(buf))?;

        buf.put(&self.message.body.inner);
        for _ in 0..size.body_padding() {
            buf.put_u8(0);
        }
        Ok(())
    }
}

impl Unbuffer for SequencedMessage<GenericBody> {
    /// Deserialize from a buffer.
    fn unbuffer_ref(buf: &mut Bytes) -> Result<SequencedMessage<GenericBody>> {
        let initial_remaining = buf.len();
        let length_field = u32::unbuffer_ref(buf).map_exactly_err_to_at_least()?;
        let size = MessageSize::from_length_field(length_field);

        // Subtracting the length of the u32 we already unbuffered.
        let expected_remaining_bytes = size.padded_message_size() - size_of::<u32>();

        if buf.len() < expected_remaining_bytes {
            return Err(Error::NeedMoreData(BytesRequired::Exactly(
                expected_remaining_bytes - buf.len(),
            )));
        }
        let time = TimeVal::unbuffer_ref(buf)?;
        let sender = SenderId::unbuffer_ref(buf)?;
        let message_type = TypeId::unbuffer_ref(buf)?;
        let sequence_number = SequenceNumber::unbuffer_ref(buf)?;

        // Assert that handling the sequence number meant we're now aligned again.
        assert_eq!((initial_remaining - buf.len()) % ALIGN, 0);

        let body;
        {
            let mut body_buf = buf.split_to(size.unpadded_body_size());
            body = GenericBody::unbuffer_ref(&mut body_buf).map_exactly_err_to_at_least()?;
            assert_eq!(body_buf.len(), 0);
        }

        // drop padding bytes
        let _ = buf.split_to(size.body_padding());
        Ok(SequencedMessage::new(
            Some(time),
            message_type,
            sender,
            body,
            sequence_number,
        ))
    }
}

impl Unbuffer for GenericBody {
    fn unbuffer_ref(buf: &mut Bytes) -> Result<GenericBody> {
        let my_buf = buf.clone();
        buf.advance(my_buf.len());
        Ok(GenericBody::new(my_buf))
    }
}

impl BufferSize for GenericBody {
    fn buffer_size(&self) -> usize {
        self.inner.len()
    }
}
impl Buffer for GenericBody {
    fn buffer_ref<T: BufMut>(&self, buf: &mut T) -> EmptyResult {
        if buf.remaining_mut() < self.inner.len() {
            return Err(Error::OutOfBuffer);
        }
        buf.put(self.inner.clone());
        Ok(())
    }
}

pub fn unbuffer_typed_message_body<T: Unbuffer + TypedMessageBody>(
    msg: &GenericMessage,
) -> Result<Message<T>> {
    let typed_msg: Message<T> = Message::try_from_generic(msg)?;
    Ok(typed_msg)
}

#[deprecated]
pub fn make_message_body_generic<T: Buffer + TypedMessageBody>(
    msg: Message<T>,
) -> Result<GenericMessage> {
    msg.try_into_generic()
}

#[deprecated]
pub fn make_sequenced_message_body_generic<T: Buffer + TypedMessageBody>(
    msg: SequencedMessage<T>,
) -> Result<SequencedGenericMessage> {
    let seq = msg.sequence_number;
    let generic_msg = msg.message.try_into_generic()?;
    Ok(generic_msg.into_sequenced_message(seq))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConstantBufferSize;
    #[test]
    fn constant() {
        // The size field is a u32.
        let len_size = size_of::<u32>();
        let computed_size = len_size
            + TimeVal::constant_buffer_size()
            + SenderId::constant_buffer_size()
            + TypeId::constant_buffer_size();
        assert_eq!(MessageSize::UNPADDED_HEADER_SIZE,
            computed_size,
            "The constant for header size should match the actual size of the fields in the header.");
        assert_eq!(
            (MessageSize::UNPADDED_HEADER_SIZE + SequenceNumber::constant_buffer_size()) % ALIGN,
            0,
            "The sequence number should make our header need no additional padding."
        );
    }

    #[test]
    fn sizes() {
        // Based on the initial "VRPN Control" sender ID message
        assert_eq!(
            MessageSize::from_unpadded_body_size(17).unpadded_body_size(),
            17
        );
        assert_eq!(
            MessageSize::from_unpadded_body_size(17).padded_body_size(),
            24
        );
        assert_eq!(MessageSize::UNPADDED_HEADER_SIZE, 20);
        assert_eq!(
            MessageSize::from_unpadded_body_size(17).padded_message_size(),
            48
        );
        assert_eq!(MessageSize::from_unpadded_body_size(17).length_field(), 41);
        assert_eq!(MessageSize::from_length_field(41).length_field(), 41);
        assert_eq!(MessageSize::from_length_field(41).unpadded_body_size(), 17);

        // Based on the second message, which is closer to the edge and thus breaks things.
        assert_eq!(
            MessageSize::from_unpadded_body_size(13),
            MessageSize::from_length_field(0x25)
        );
        assert_eq!(
            MessageSize::from_unpadded_body_size(13).padded_body_size(),
            16
        );
        assert_eq!(
            MessageSize::from_unpadded_body_size(13).length_field(),
            24 + 13
        );
        assert_eq!(MessageSize::from_length_field(37).length_field(), 37);

        assert_eq!(MessageSize::from_length_field(37).padded_message_size(), 40);
    }
    proptest! {
        #[test]
        fn length_field_matches(len in 0u32..10000) {
            let len = len as usize;
            prop_assert_eq!(
            MessageSize::from_unpadded_body_size(len).length_field(),
            transcribed_padding_function(len).len_field);
        }

        #[test]
        fn total_length_matches(len in 0u32..10000) {
            let len = len as usize;
            prop_assert_eq!(
            MessageSize::from_unpadded_body_size(len).padded_message_size(),
            transcribed_padding_function(len).total_len);
        }

        #[test]
        fn roundtrip(len in 20u32..10000)  {
            MessageSize::from_length_field(len).length_field() == len
        }
    }

    struct Lengths {
        header_len: usize,
        total_len: usize,
        len_field: u32,
    }

    /// This is a relatively literal translation of the size computations in vrpn_Endpoint::marshall_message,
    /// used as ground truth in testing.
    fn transcribed_padding_function(len: usize) -> Lengths {
        let mut ceil_len = len;
        if (len % ALIGN) != 0 {
            ceil_len += ALIGN - len % ALIGN;
        }

        let mut header_len = 5 * std::mem::size_of::<i32>();
        if (header_len % ALIGN) != 0 {
            header_len += ALIGN - header_len % ALIGN;
        }
        let total_len = header_len + ceil_len;
        Lengths {
            header_len,
            total_len,
            len_field: (header_len + len) as u32,
        }
    }
}
