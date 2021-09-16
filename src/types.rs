// Copyright 2018, Collabora, Ltd.
// SPDX-License-Identifier: BSL-1.0
// Author: Ryan A. Pavlik <ryan.pavlik@collabora.com>

use crate::constants;
use bytes::Bytes;
use cgmath::Vector3;

/// Type wrapped by the various Id types - chosen to match VRPN C++.
pub type IdType = i32;

pub const MAX_VEC_USIZE: usize = (IdType::max_value() - 2) as usize;

pub trait TypeSafeId: Copy + Clone + Eq + PartialEq + Ord + PartialOrd {
    fn get(&self) -> IdType;
    fn new(val: IdType) -> Self;
}
pub trait IntoId: TypeSafeId {
    /// Base ID type. Self in the case of BaseTypeSafeId, otherwise the thing that's being wrapped.
    type BaseId: BaseTypeSafeId;
    fn into_id(self) -> Self::BaseId;
}
pub trait BaseTypeSafeId:
    TypeSafeId + Clone + Copy + std::fmt::Debug + PartialEq + Eq + BaseTypeSafeIdName
{
    fn description_type() -> TypeId;
}

pub trait BaseTypeSafeIdName
where
    Self::Name: Into<Bytes>,
{
    type Name;
}

/// Local-side ID in the translation table
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LocalId<T: BaseTypeSafeId>(pub T);

/// Remote-side ID in the translation table
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RemoteId<T: BaseTypeSafeId>(pub T);

/// ID for a message type
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TypeId(pub IdType);

/// ID for a sender
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SenderId(pub IdType);

impl<T: BaseTypeSafeId> TypeSafeId for LocalId<T> {
    fn get(&self) -> IdType {
        self.0.get()
    }
    fn new(val: IdType) -> LocalId<T> {
        LocalId(T::new(val))
    }
}

impl<T: BaseTypeSafeId> IntoId for T {
    type BaseId = T;
    fn into_id(self) -> Self::BaseId {
        self
    }
}

impl<T: BaseTypeSafeId> IntoId for LocalId<T> {
    type BaseId = T;
    fn into_id(self) -> Self::BaseId {
        self.0
    }
}
impl<T: BaseTypeSafeId> IntoId for RemoteId<T> {
    type BaseId = T;
    fn into_id(self) -> Self::BaseId {
        self.0
    }
}
impl<T: BaseTypeSafeId> TypeSafeId for RemoteId<T> {
    fn get(&self) -> IdType {
        self.0.get()
    }
    fn new(val: IdType) -> RemoteId<T> {
        RemoteId(T::new(val))
    }
}

impl TypeSafeId for TypeId {
    fn get(&self) -> IdType {
        self.0
    }
    fn new(val: IdType) -> TypeId {
        TypeId(val)
    }
}

impl BaseTypeSafeId for TypeId {
    fn description_type() -> TypeId {
        constants::TYPE_DESCRIPTION
    }
}

impl TypeId {
    /// Identifies if this is a system message.
    ///
    /// If false, it's a normal (user) message.
    pub fn is_system_message(&self) -> bool {
        self.0 < 0
    }
}

impl BaseTypeSafeIdName for TypeId {
    type Name = TypeName;
}

impl TypeSafeId for SenderId {
    fn get(&self) -> IdType {
        self.0
    }
    fn new(val: IdType) -> SenderId {
        SenderId(val)
    }
}

impl BaseTypeSafeId for SenderId {
    fn description_type() -> TypeId {
        constants::SENDER_DESCRIPTION
    }
}

impl BaseTypeSafeIdName for SenderId {
    type Name = SenderName;
}

pub fn id_filter_matches<T>(filter: Option<T>, other: T) -> bool
where
    T: TypeSafeId,
{
    match filter {
        None => true,
        Some(i) => i == other,
    }
}
bitflags! {
    pub struct ClassOfService : u32 {
        const Reliable = (1 << 0);
        const FixedLatency = (1 << 1);
        const LowLatency = (1 << 2);
        const FixedThroughput = (1 << 3);
        const HighThroughput = (1 << 4);
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct StaticSenderName(pub &'static [u8]);

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct SenderName(pub Bytes);

impl From<&'static [u8]> for SenderName {
    fn from(val: &'static [u8]) -> SenderName {
        SenderName(Bytes::from_static(val))
    }
}

impl From<&'static [u8]> for StaticSenderName {
    fn from(val: &'static [u8]) -> StaticSenderName {
        StaticSenderName(val)
    }
}

impl From<StaticSenderName> for SenderName {
    fn from(val: StaticSenderName) -> SenderName {
        SenderName(Bytes::from(val))
    }
}

impl From<StaticSenderName> for Bytes {
    fn from(val: StaticSenderName) -> Bytes {
        Bytes::from_static(val.0)
    }
}

impl From<SenderName> for Bytes {
    fn from(val: SenderName) -> Bytes {
        val.0
    }
}

impl std::cmp::PartialEq<SenderName> for StaticSenderName {
    fn eq(&self, other: &SenderName) -> bool {
        Bytes::from_static(self.0) == other.0
    }
}

impl std::cmp::PartialEq<StaticSenderName> for SenderName {
    fn eq(&self, other: &StaticSenderName) -> bool {
        self.0 == Bytes::from_static(other.0)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct StaticTypeName(pub &'static [u8]);

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct TypeName(pub Bytes);

impl From<&'static [u8]> for TypeName {
    fn from(val: &'static [u8]) -> TypeName {
        TypeName(Bytes::from_static(val))
    }
}

impl From<&'static [u8]> for StaticTypeName {
    fn from(val: &'static [u8]) -> StaticTypeName {
        StaticTypeName(val)
    }
}

impl From<StaticTypeName> for TypeName {
    fn from(val: StaticTypeName) -> TypeName {
        TypeName(Bytes::from(val))
    }
}

impl From<StaticTypeName> for Bytes {
    fn from(val: StaticTypeName) -> Bytes {
        Bytes::from_static(val.0)
    }
}

impl From<TypeName> for Bytes {
    fn from(val: TypeName) -> Bytes {
        val.0
    }
}

impl std::cmp::PartialEq<TypeName> for StaticTypeName {
    fn eq(&self, other: &TypeName) -> bool {
        Bytes::from_static(self.0) == other.0
    }
}

impl std::cmp::PartialEq<StaticTypeName> for TypeName {
    fn eq(&self, other: &StaticTypeName) -> bool {
        self.0 == Bytes::from_static(other.0)
    }
}

/// Sequence number - not used on receive side, only used for sniffers (?)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SequenceNumber(pub u32);

/// Sensor ID for trackers.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Sensor(pub i32);

// pub type Quat = Quaternion<f64>;
pub type Vec3 = Vector3<f64>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub s: f64,
    pub v: Vec3,
}

impl Quat {
    pub fn from_sv(s: f64, v: Vector3<f64>) -> Quat {
        Quat { s, v }
    }

    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Quat {
        Quat {
            s: w,
            v: Vec3::new(x, y, z),
        }
    }
    pub fn identity() -> Quat {
        Quat {
            s: 1.0,
            v: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

impl From<cgmath::Quaternion<f64>> for Quat {
    fn from(q: cgmath::Quaternion<f64>) -> Self {
        Quat { s: q.s, v: q.v }
    }
}

impl From<Quat> for cgmath::Quaternion<f64> {
    fn from(q: Quat) -> Self {
        cgmath::Quaternion::from_sv(q.s, q.v)
    }
}

pub(crate) enum RangedId {
    BelowZero(IdType),
    InArray(IdType),
    AboveArray(IdType),
}

/// Categorize an ID into either below array, in array, or above array.
///
/// Typically, calling code will then match on the result and make one or more
/// of the variants produce an error. However, which ones are errors vary between
/// functions.
pub(crate) fn determine_id_range<T: BaseTypeSafeId>(id: T, len: usize) -> RangedId {
    let raw = id.get();
    if raw < 0 {
        RangedId::BelowZero(raw)
    } else {
        let index = raw as usize;
        if index < len {
            RangedId::InArray(raw)
        } else {
            RangedId::AboveArray(raw)
        }
    }
}
