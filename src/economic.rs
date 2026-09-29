//! Compact economic activity ABI for AtomVM / edge execution.
//!
//! The one-byte opcode identifies only the economic verb. Objects, values,
//! provenance and authority remain external structured data. This module is a
//! projection of the canonical ex4pm economic ISA and MUST NOT allocate new
//! meanings to reserved bytes.

use core::convert::TryFrom;

pub const UNKNOWN_BYTE: u8 = 0x00;
pub const ESCAPE_BYTE: u8 = 0xFF;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EconomicCategory {
    Null,
    Market,
    Transaction,
    PaymentSettlement,
    Logistics,
    ContractRights,
    ProductionService,
    AccountingFinance,
    GovernanceAuthority,
    Extensions,
    Escape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EconomicOpcode {
    Unknown = 0x00,

    Quote = 0x01,
    Offer = 0x02,
    Bid = 0x03,
    Ask = 0x04,
    Discover = 0x05,

    Order = 0x20,
    Fill = 0x21,
    Sale = 0x22,
    Purchase = 0x23,
    Return = 0x24,
    Cancel = 0x25,
    Exchange = 0x26,

    Invoice = 0x40,
    Pay = 0x41,
    Settle = 0x42,
    Refund = 0x43,
    AuthorizePayment = 0x44,
    CapturePayment = 0x45,

    Ship = 0x60,
    Deliver = 0x61,
    Receive = 0x62,
    Move = 0x63,
    Store = 0x64,
    Consume = 0x65,

    Sign = 0x80,
    License = 0x81,
    Subscribe = 0x82,
    Renew = 0x83,
    Terminate = 0x84,
    AssignRight = 0x85,

    Manufacture = 0xA0,
    Design = 0xA1,
    Produce = 0xA2,
    ProvideService = 0xA3,
    Prove = 0xA4,
    Inspect = 0xA5,
    Accept = 0xA6,

    Accrue = 0xC0,
    RecognizeRevenue = 0xC1,
    RecognizeExpense = 0xC2,
    Capitalize = 0xC3,
    Depreciate = 0xC4,
    RealizeValue = 0xC5,
    Allocate = 0xC6,

    Observe = 0xE0,
    Authorize = 0xE1,
    Attest = 0xE2,
    Approve = 0xE3,
    Reject = 0xE4,
    Dispute = 0xE5,
    Resolve = 0xE6,

    /// Escape marker. Use `EconomicFrame::Extended` when a semantic payload is present.
    Extended = 0xFF,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EconomicFrame<'a> {
    Fixed(EconomicOpcode),
    Extended(&'a [u8]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EconomicCodecError {
    Empty,
    Unassigned(u8),
    MissingExtendedSemanticId,
    TrailingBytes,
}

impl EconomicOpcode {
    #[inline]
    pub const fn byte(self) -> u8 {
        self as u8
    }

    pub const fn category(self) -> EconomicCategory {
        category_for_byte(self as u8)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Quote => "quote",
            Self::Offer => "offer",
            Self::Bid => "bid",
            Self::Ask => "ask",
            Self::Discover => "discover",
            Self::Order => "order",
            Self::Fill => "fill",
            Self::Sale => "sale",
            Self::Purchase => "purchase",
            Self::Return => "return",
            Self::Cancel => "cancel",
            Self::Exchange => "exchange",
            Self::Invoice => "invoice",
            Self::Pay => "pay",
            Self::Settle => "settle",
            Self::Refund => "refund",
            Self::AuthorizePayment => "authorize_payment",
            Self::CapturePayment => "capture_payment",
            Self::Ship => "ship",
            Self::Deliver => "deliver",
            Self::Receive => "receive",
            Self::Move => "move",
            Self::Store => "store",
            Self::Consume => "consume",
            Self::Sign => "sign",
            Self::License => "license",
            Self::Subscribe => "subscribe",
            Self::Renew => "renew",
            Self::Terminate => "terminate",
            Self::AssignRight => "assign_right",
            Self::Manufacture => "manufacture",
            Self::Design => "design",
            Self::Produce => "produce",
            Self::ProvideService => "provide_service",
            Self::Prove => "prove",
            Self::Inspect => "inspect",
            Self::Accept => "accept",
            Self::Accrue => "accrue",
            Self::RecognizeRevenue => "recognize_revenue",
            Self::RecognizeExpense => "recognize_expense",
            Self::Capitalize => "capitalize",
            Self::Depreciate => "depreciate",
            Self::RealizeValue => "realize_value",
            Self::Allocate => "allocate",
            Self::Observe => "observe",
            Self::Authorize => "authorize",
            Self::Attest => "attest",
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Dispute => "dispute",
            Self::Resolve => "resolve",
            Self::Extended => "extended",
        }
    }
}

impl From<EconomicOpcode> for u8 {
    fn from(value: EconomicOpcode) -> Self {
        value.byte()
    }
}

impl TryFrom<u8> for EconomicOpcode {
    type Error = EconomicCodecError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let opcode = match value {
            0x00 => Self::Unknown,
            0x01 => Self::Quote,
            0x02 => Self::Offer,
            0x03 => Self::Bid,
            0x04 => Self::Ask,
            0x05 => Self::Discover,
            0x20 => Self::Order,
            0x21 => Self::Fill,
            0x22 => Self::Sale,
            0x23 => Self::Purchase,
            0x24 => Self::Return,
            0x25 => Self::Cancel,
            0x26 => Self::Exchange,
            0x40 => Self::Invoice,
            0x41 => Self::Pay,
            0x42 => Self::Settle,
            0x43 => Self::Refund,
            0x44 => Self::AuthorizePayment,
            0x45 => Self::CapturePayment,
            0x60 => Self::Ship,
            0x61 => Self::Deliver,
            0x62 => Self::Receive,
            0x63 => Self::Move,
            0x64 => Self::Store,
            0x65 => Self::Consume,
            0x80 => Self::Sign,
            0x81 => Self::License,
            0x82 => Self::Subscribe,
            0x83 => Self::Renew,
            0x84 => Self::Terminate,
            0x85 => Self::AssignRight,
            0xA0 => Self::Manufacture,
            0xA1 => Self::Design,
            0xA2 => Self::Produce,
            0xA3 => Self::ProvideService,
            0xA4 => Self::Prove,
            0xA5 => Self::Inspect,
            0xA6 => Self::Accept,
            0xC0 => Self::Accrue,
            0xC1 => Self::RecognizeRevenue,
            0xC2 => Self::RecognizeExpense,
            0xC3 => Self::Capitalize,
            0xC4 => Self::Depreciate,
            0xC5 => Self::RealizeValue,
            0xC6 => Self::Allocate,
            0xE0 => Self::Observe,
            0xE1 => Self::Authorize,
            0xE2 => Self::Attest,
            0xE3 => Self::Approve,
            0xE4 => Self::Reject,
            0xE5 => Self::Dispute,
            0xE6 => Self::Resolve,
            0xFF => Self::Extended,
            other => return Err(EconomicCodecError::Unassigned(other)),
        };
        Ok(opcode)
    }
}

pub const fn category_for_byte(byte: u8) -> EconomicCategory {
    match byte {
        0x00 => EconomicCategory::Null,
        0x01..=0x1F => EconomicCategory::Market,
        0x20..=0x3F => EconomicCategory::Transaction,
        0x40..=0x5F => EconomicCategory::PaymentSettlement,
        0x60..=0x7F => EconomicCategory::Logistics,
        0x80..=0x9F => EconomicCategory::ContractRights,
        0xA0..=0xBF => EconomicCategory::ProductionService,
        0xC0..=0xDF => EconomicCategory::AccountingFinance,
        0xE0..=0xEF => EconomicCategory::GovernanceAuthority,
        0xF0..=0xFE => EconomicCategory::Extensions,
        0xFF => EconomicCategory::Escape,
    }
}

/// Decode exactly one economic frame without allocation.
///
/// Fixed common-path activities occupy exactly one byte. `0xFF` borrows the
/// remaining bytes as the lossless semantic identifier for an extended activity.
pub fn decode(bytes: &[u8]) -> Result<EconomicFrame<'_>, EconomicCodecError> {
    let (&head, tail) = bytes.split_first().ok_or(EconomicCodecError::Empty)?;

    if head == ESCAPE_BYTE {
        if tail.is_empty() {
            Err(EconomicCodecError::MissingExtendedSemanticId)
        } else {
            Ok(EconomicFrame::Extended(tail))
        }
    } else if !tail.is_empty() {
        Err(EconomicCodecError::TrailingBytes)
    } else {
        EconomicOpcode::try_from(head).map(EconomicFrame::Fixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSIGNED: &[EconomicOpcode] = &[
        EconomicOpcode::Unknown,
        EconomicOpcode::Quote, EconomicOpcode::Offer, EconomicOpcode::Bid,
        EconomicOpcode::Ask, EconomicOpcode::Discover,
        EconomicOpcode::Order, EconomicOpcode::Fill, EconomicOpcode::Sale,
        EconomicOpcode::Purchase, EconomicOpcode::Return, EconomicOpcode::Cancel,
        EconomicOpcode::Exchange,
        EconomicOpcode::Invoice, EconomicOpcode::Pay, EconomicOpcode::Settle,
        EconomicOpcode::Refund, EconomicOpcode::AuthorizePayment,
        EconomicOpcode::CapturePayment,
        EconomicOpcode::Ship, EconomicOpcode::Deliver, EconomicOpcode::Receive,
        EconomicOpcode::Move, EconomicOpcode::Store, EconomicOpcode::Consume,
        EconomicOpcode::Sign, EconomicOpcode::License, EconomicOpcode::Subscribe,
        EconomicOpcode::Renew, EconomicOpcode::Terminate, EconomicOpcode::AssignRight,
        EconomicOpcode::Manufacture, EconomicOpcode::Design, EconomicOpcode::Produce,
        EconomicOpcode::ProvideService, EconomicOpcode::Prove, EconomicOpcode::Inspect,
        EconomicOpcode::Accept,
        EconomicOpcode::Accrue, EconomicOpcode::RecognizeRevenue,
        EconomicOpcode::RecognizeExpense, EconomicOpcode::Capitalize,
        EconomicOpcode::Depreciate, EconomicOpcode::RealizeValue,
        EconomicOpcode::Allocate,
        EconomicOpcode::Observe, EconomicOpcode::Authorize, EconomicOpcode::Attest,
        EconomicOpcode::Approve, EconomicOpcode::Reject, EconomicOpcode::Dispute,
        EconomicOpcode::Resolve,
    ];

    #[test]
    fn fixed_common_path_is_exactly_one_byte_and_round_trips() {
        for opcode in ASSIGNED {
            let byte = opcode.byte();
            assert_eq!(decode(&[byte]), Ok(EconomicFrame::Fixed(*opcode)));
            assert_eq!(EconomicOpcode::try_from(byte), Ok(*opcode));
        }
    }

    #[test]
    fn reserved_bytes_refuse_instead_of_acquiring_local_meanings() {
        assert_eq!(EconomicOpcode::try_from(0x06), Err(EconomicCodecError::Unassigned(0x06)));
        assert_eq!(EconomicOpcode::try_from(0x27), Err(EconomicCodecError::Unassigned(0x27)));
        assert_eq!(EconomicOpcode::try_from(0xF0), Err(EconomicCodecError::Unassigned(0xF0)));
    }

    #[test]
    fn unknown_and_escape_boundaries_are_stable() {
        assert_eq!(EconomicOpcode::Unknown.byte(), UNKNOWN_BYTE);
        assert_eq!(EconomicOpcode::Unknown.category(), EconomicCategory::Null);
        assert_eq!(EconomicOpcode::Extended.byte(), ESCAPE_BYTE);
        assert_eq!(EconomicOpcode::Extended.category(), EconomicCategory::Escape);
    }

    #[test]
    fn extended_semantic_identifier_is_borrowed_losslessly() {
        let semantic_id = b"urn:example:economic:custom-action";
        let mut frame = [0u8; 64];
        frame[0] = ESCAPE_BYTE;
        frame[1..1 + semantic_id.len()].copy_from_slice(semantic_id);
        assert_eq!(
            decode(&frame[..1 + semantic_id.len()]),
            Ok(EconomicFrame::Extended(semantic_id))
        );
    }

    #[test]
    fn fixed_frame_rejects_trailing_payload() {
        assert_eq!(decode(&[0x41, 0x00]), Err(EconomicCodecError::TrailingBytes));
    }
}
