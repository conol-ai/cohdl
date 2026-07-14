//! RFC-001: Units-as-types.
//!
//! A closed set of eleven primitive unit types (ten from RFC-001 + Length
//! from RFC-018). Zero implicit coercion between
//! unit types or from bare numbers. The (unit × allowed-prefix) table below is
//! the normative grammar table from the Language Specification (note 10):
//!
//! | Unit         | Symbol | Prefixes        | Signed |
//! |--------------|--------|-----------------|--------|
//! | Voltage      | V      | p n u m k M G   | no     |
//! | Capacitance  | F      | p n u           | no     |
//! | Resistance   | ohm    | p n u m k M G   | no     |
//! | Current      | A      | p n u m k M G   | no     |
//! | Frequency    | Hz     | k M G           | no     |
//! | Time         | s      | p n u m         | no     |
//! | Inductance   | H      | p n u m         | no     |
//! | Power        | W      | u m k           | no     |
//! | Temperature  | C      | (none)          | YES    |
//! | Tolerance    | %      | (none)          | no     |
//! | Length       | mm     | (none)          | YES    |
//!
//! "standard" prefixes in the spec table are pinned here as `p n u m k M G`.
//! An unprefixed literal (`5V`, `330ohm`) is always valid for its unit.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitType {
    Voltage,
    Capacitance,
    Resistance,
    Current,
    Frequency,
    Time,
    Inductance,
    Power,
    Temperature,
    Tolerance,
    /// RFC-018: physical dimensions for pads/footprints (`0.3mm`,
    /// `-1.5mm` — signed, coordinates are offsets). The eleventh unit
    /// type; `mm` is the whole suffix (no SI prefixes).
    Length,
}

pub const ALL_UNIT_TYPES: [UnitType; 11] = [
    UnitType::Voltage,
    UnitType::Capacitance,
    UnitType::Resistance,
    UnitType::Current,
    UnitType::Frequency,
    UnitType::Time,
    UnitType::Inductance,
    UnitType::Power,
    UnitType::Temperature,
    UnitType::Tolerance,
    UnitType::Length,
];

impl UnitType {
    /// The type name as written in `.cohdl` source (e.g. in a spec field type
    /// or generic bound).
    pub fn type_name(self) -> &'static str {
        match self {
            UnitType::Voltage => "Voltage",
            UnitType::Capacitance => "Capacitance",
            UnitType::Resistance => "Resistance",
            UnitType::Current => "Current",
            UnitType::Frequency => "Frequency",
            UnitType::Time => "Time",
            UnitType::Inductance => "Inductance",
            UnitType::Power => "Power",
            UnitType::Temperature => "Temperature",
            UnitType::Tolerance => "Tolerance",
            UnitType::Length => "Length",
        }
    }

    /// The canonical ASCII literal suffix (one spelling per unit — `ohm`
    /// never `Ω`, `C` never `°C`).
    pub fn symbol(self) -> &'static str {
        match self {
            UnitType::Voltage => "V",
            UnitType::Capacitance => "F",
            UnitType::Resistance => "ohm",
            UnitType::Current => "A",
            UnitType::Frequency => "Hz",
            UnitType::Time => "s",
            UnitType::Inductance => "H",
            UnitType::Power => "W",
            UnitType::Temperature => "C",
            UnitType::Tolerance => "%",
            UnitType::Length => "mm",
        }
    }

    pub fn from_type_name(name: &str) -> Option<UnitType> {
        ALL_UNIT_TYPES
            .iter()
            .copied()
            .find(|u| u.type_name() == name)
    }

    fn from_symbol(sym: &str) -> Option<UnitType> {
        ALL_UNIT_TYPES.iter().copied().find(|u| u.symbol() == sym)
    }

    /// Allowed SI prefixes for this unit (empty = takes no prefix at all).
    pub fn allowed_prefixes(self) -> &'static [SiPrefix] {
        use SiPrefix::*;
        match self {
            UnitType::Voltage | UnitType::Resistance | UnitType::Current => {
                &[Pico, Nano, Micro, Milli, Kilo, Mega, Giga]
            }
            UnitType::Capacitance => &[Pico, Nano, Micro],
            UnitType::Frequency => &[Kilo, Mega, Giga],
            UnitType::Time | UnitType::Inductance => &[Pico, Nano, Micro, Milli],
            UnitType::Power => &[Micro, Milli, Kilo],
            UnitType::Temperature | UnitType::Tolerance | UnitType::Length => &[],
        }
    }

    /// Temperature (below-zero ratings) and Length (signed pad-placement
    /// coordinates, RFC-018) may carry a leading `-`.
    pub fn allows_negative(self) -> bool {
        matches!(self, UnitType::Temperature | UnitType::Length)
    }

    /// RFC-001's (unit × allowed-prefix) table row, formatted for editor
    /// hover (RFC-014 inherits RFC-001's "surfaced via LSP hover" ask).
    pub fn prefix_table_help(self) -> String {
        let prefixes = self.allowed_prefixes();
        if prefixes.is_empty() {
            format!("- suffix `{}` takes no SI prefix", self.symbol())
        } else {
            format!(
                "- allowed prefixes on `{}`: {}",
                self.symbol(),
                prefixes
                    .iter()
                    .map(|p| format!("`{}`", p.letter()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

impl fmt::Display for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.type_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SiPrefix {
    Pico,
    Nano,
    Micro,
    Milli,
    Kilo,
    Mega,
    Giga,
}

impl SiPrefix {
    pub fn letter(self) -> char {
        match self {
            SiPrefix::Pico => 'p',
            SiPrefix::Nano => 'n',
            SiPrefix::Micro => 'u',
            SiPrefix::Milli => 'm',
            SiPrefix::Kilo => 'k',
            SiPrefix::Mega => 'M',
            SiPrefix::Giga => 'G',
        }
    }

    fn from_letter(c: char) -> Option<SiPrefix> {
        Some(match c {
            'p' => SiPrefix::Pico,
            'n' => SiPrefix::Nano,
            'u' => SiPrefix::Micro,
            'm' => SiPrefix::Milli,
            'k' => SiPrefix::Kilo,
            'M' => SiPrefix::Mega,
            'G' => SiPrefix::Giga,
            _ => return None,
        })
    }

    /// Power-of-ten exponent.
    pub fn exponent(self) -> i32 {
        match self {
            SiPrefix::Pico => -12,
            SiPrefix::Nano => -9,
            SiPrefix::Micro => -6,
            SiPrefix::Milli => -3,
            SiPrefix::Kilo => 3,
            SiPrefix::Mega => 6,
            SiPrefix::Giga => 9,
        }
    }
}

/// A concrete unit-typed value.
///
/// Stored exactly, as an integer count of 10^-15 of the unit's base (femto-
/// units): `3.3V` = 3_300_000_000_000_000. This gives exact, total ordering
/// for same-unit comparison (the only arithmetic the language defines) and a
/// byte-stable canonical rendering. `text` preserves the literal exactly as
/// written for display/netlist output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitValue {
    pub unit: UnitType,
    /// Value in 10^-15 units. Negative only for Temperature.
    pub femto: i128,
    /// The literal as written in source, e.g. `100nF`.
    pub text: String,
}

impl UnitValue {
    pub fn cmp_same_unit(&self, other: &UnitValue) -> std::cmp::Ordering {
        debug_assert_eq!(self.unit, other.unit);
        self.femto.cmp(&other.femto)
    }

    /// True when this `Length` (or any) value is small enough for the
    /// geometry emitters' corner arithmetic to stay within `i128` (review
    /// R5-5). Checked at pad/footprint validation so an out-of-range value
    /// is a clean diagnostic, never a panic mid-build.
    pub fn length_in_geom_range(&self) -> bool {
        self.femto.unsigned_abs() <= crate::emit::geom::MAX_GEOM_FEMTO as u128
    }
}

impl fmt::Display for UnitValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Why a numeric-literal suffix failed to parse as a unit literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitLexError {
    /// Suffix isn't any unit symbol, with or without a leading prefix.
    UnknownSuffix { suffix: String },
    /// Valid unit, but the prefix isn't in the allowed table for it
    /// (includes any prefix on Temperature/Tolerance, which take none).
    PrefixNotAllowed { unit: UnitType, prefix: SiPrefix },
    /// Leading `-` on a unit other than Temperature.
    NegativeNotAllowed { unit: UnitType },
    /// Mantissa has too many decimal digits to represent exactly.
    TooPrecise,
    /// Magnitude overflows the exact representation.
    Overflow,
}

/// Parse the suffix of a unit literal (everything after the number) into
/// (prefix, unit). Deterministic: an exact symbol match wins; otherwise the
/// first character must be an SI prefix letter and the remainder an exact
/// symbol. (No symbol equals a prefix letter + another symbol, so the two
/// cases never overlap.)
pub fn parse_suffix(suffix: &str) -> Result<(Option<SiPrefix>, UnitType), UnitLexError> {
    if let Some(u) = UnitType::from_symbol(suffix) {
        return Ok((None, u));
    }
    let mut chars = suffix.chars();
    if let Some(first) = chars.next() {
        if let Some(prefix) = SiPrefix::from_letter(first) {
            if let Some(u) = UnitType::from_symbol(chars.as_str()) {
                return Ok((Some(prefix), u));
            }
        }
    }
    Err(UnitLexError::UnknownSuffix {
        suffix: suffix.to_string(),
    })
}

/// Build a [`UnitValue`] from the parsed pieces of a literal.
///
/// `mantissa` is the digits as written (e.g. `"3.3"`, `"100"`, `"0.5"`),
/// `negative` reflects a leading `-`.
pub fn make_value(
    negative: bool,
    mantissa: &str,
    prefix: Option<SiPrefix>,
    unit: UnitType,
    original_text: &str,
) -> Result<UnitValue, UnitLexError> {
    if let Some(p) = prefix {
        if !unit.allowed_prefixes().contains(&p) {
            return Err(UnitLexError::PrefixNotAllowed { unit, prefix: p });
        }
    }
    if negative && !unit.allows_negative() {
        return Err(UnitLexError::NegativeNotAllowed { unit });
    }

    let (int_part, frac_part) = match mantissa.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mantissa, ""),
    };
    // Total scale: femto (1e-15) adjusted by the SI prefix.
    let exp = 15 + prefix.map_or(0, |p| p.exponent());
    let frac_digits = frac_part.len() as i32;
    if frac_digits > exp {
        return Err(UnitLexError::TooPrecise);
    }
    let digits: String = [int_part, frac_part].concat();
    let base: i128 = digits.parse().map_err(|_| UnitLexError::Overflow)?;
    let scale = exp - frac_digits;
    let mut value = base;
    for _ in 0..scale {
        value = value.checked_mul(10).ok_or(UnitLexError::Overflow)?;
    }
    if negative {
        value = -value;
    }
    Ok(UnitValue {
        unit,
        femto: value,
        text: original_text.to_string(),
    })
}

/// The full unit table as a human-readable reference block (used by
/// diagnostics that need to show valid options).
pub fn prefix_table_help(unit: UnitType) -> String {
    let prefixes = unit.allowed_prefixes();
    if prefixes.is_empty() {
        format!("`{}` takes no SI prefix at all", unit.type_name())
    } else {
        let list: Vec<String> = prefixes.iter().map(|p| p.letter().to_string()).collect();
        format!(
            "valid prefixes for `{}` are: {} (e.g. `10{}{}`)",
            unit.type_name(),
            list.join(" "),
            prefixes[0].letter(),
            unit.symbol()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> UnitValue {
        // Split text into number and suffix for the test helper.
        let negative = text.starts_with('-');
        let rest = text.strip_prefix('-').unwrap_or(text);
        let split = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap();
        let (num, suffix) = rest.split_at(split);
        let (prefix, unit) = parse_suffix(suffix).unwrap();
        make_value(negative, num, prefix, unit, text).unwrap()
    }

    #[test]
    fn parses_all_ten_units() {
        assert_eq!(v("3.3V").unit, UnitType::Voltage);
        assert_eq!(v("100nF").unit, UnitType::Capacitance);
        assert_eq!(v("10kohm").unit, UnitType::Resistance);
        assert_eq!(v("500mA").unit, UnitType::Current);
        assert_eq!(v("16MHz").unit, UnitType::Frequency);
        assert_eq!(v("10ms").unit, UnitType::Time);
        assert_eq!(v("10uH").unit, UnitType::Inductance);
        assert_eq!(v("250mW").unit, UnitType::Power);
        assert_eq!(v("85C").unit, UnitType::Temperature);
        assert_eq!(v("1%").unit, UnitType::Tolerance);
    }

    #[test]
    fn exact_values() {
        assert_eq!(v("3.3V").femto, 3_300_000_000_000_000);
        assert_eq!(v("100nF").femto, 100_000_000);
        assert_eq!(v("0.5%").femto, 500_000_000_000_000);
        assert_eq!(v("-40C").femto, -40_000_000_000_000_000);
        assert_eq!(v("10kohm").femto, 10_000_000_000_000_000_000);
    }

    #[test]
    fn comparison_is_exact() {
        assert!(v("3.3V").cmp_same_unit(&v("5V")).is_lt());
        assert!(v("100nF").cmp_same_unit(&v("0.1uF")).is_eq());
        assert!(v("-40C").cmp_same_unit(&v("85C")).is_lt());
    }

    #[test]
    fn rejects_bad_prefixes() {
        // mF: milli not allowed for Capacitance.
        let (p, u) = parse_suffix("mF").unwrap();
        assert!(matches!(
            make_value(false, "1", p, u, "1mF"),
            Err(UnitLexError::PrefixNotAllowed { .. })
        ));
        // mC: Temperature takes no prefix.
        let (p, u) = parse_suffix("mC").unwrap();
        assert!(matches!(
            make_value(false, "1", p, u, "1mC"),
            Err(UnitLexError::PrefixNotAllowed { .. })
        ));
    }

    #[test]
    fn rejects_negative_except_temperature() {
        let (p, u) = parse_suffix("V").unwrap();
        assert!(matches!(
            make_value(true, "5", p, u, "-5V"),
            Err(UnitLexError::NegativeNotAllowed { .. })
        ));
    }

    #[test]
    fn unknown_suffix() {
        assert!(parse_suffix("X").is_err());
        assert!(parse_suffix("mX").is_err());
        // Bare prefix letter is not a unit.
        assert!(parse_suffix("m").is_err());
    }
}
