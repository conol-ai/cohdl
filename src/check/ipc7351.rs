//! RFC-021: the closed IPC-7351B footprint-naming grammar CoHDL adopts.
//!
//! A `footprint`'s OWN identifier (when its package prefix is one of the closed
//! six families below) must parse against that family's template (the closed
//! set covering CoHDL's real current hardware — extending it is a scoped
//! follow-up, same discipline as RFC-001's closed unit-type set). Dimensions
//! are encoded in hundredths of a millimetre with no decimal point or unit
//! (`50` = 0.50mm, `700` = 7.00mm) — IPC-7351B's own convention, adopted as
//! CoHDL's own footprint-naming convention. Per RFC-021's second revision this
//! is purely a naming convention for CoHDL's own native footprints; CoHDL does
//! not reference or track any third-party CAD tool's footprint library.
//!
//! RFC-021 (twice-revised): the footprint's OWN identifier IS the IPC-7351 name,
//! with IPC-7351's `-` mapped to `_` (CoHDL identifiers disallow `-`). So the
//! templates below are the identifier form (`_` where IPC-7351 writes `-`):
//!
//! | Family | Template (identifier form) |
//! |---|---|
//! | QFP  | `QFP{pitch}P{lsX}X{lsY}X{h}_{pins}{D}` |
//! | QFN  | `QFN{pins}{D}{pitch}P{bX}X{bY}[_1EP{eX}X{eY}]`  (incl. SON/VQFN) |
//! | SOIC/SOP | `SOIC{pins}P{pitch}X{ls}X{h}{D}` |
//! | SOT  | `SOT{pins}P{pitch}X{bX}X{bY}{D}` |
//! | BGA  | `BGA{pins}{C|N}{pitch}P{cols}X{rows}_{bX}X{bY}X{h}{D}` |
//! | CHIP/MELF | `CHIP_{EIA}` (e.g. `CHIP_0402`) — no density suffix |
//!
//! `D` is the density level, a closed set `{N, L, M}`. A name whose prefix is
//! none of the closed family prefixes is NOT an IPC-7351 name (out of the
//! closed set, RFC-021 Non-goals) — `parse` returns `UnknownFamily` and the
//! caller leaves it as an ordinary RFC-016 identifier, unchecked.
//!
//! Only the fields the geometry cross-check needs (`pins`, `pitch`, `has_ep`)
//! are surfaced; the descriptive dimensions are validated as well-formed
//! integers but otherwise unused (they describe, they are not checked against
//! geometry — RFC-021 Non-goals).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Qfp,
    Qfn,
    Soic,
    Sop,
    Sot,
    Bga,
    Chip,
    Melf,
}

impl Family {
    pub fn label(&self) -> &'static str {
        match self {
            Family::Qfp => "QFP",
            Family::Qfn => "QFN/SON",
            Family::Soic => "SOIC",
            Family::Sop => "SOP",
            Family::Sot => "SOT",
            Family::Bga => "BGA",
            Family::Chip => "CHIP",
            Family::Melf => "MELF",
        }
    }
}

/// A parsed, well-formed IPC-7351 footprint name. `pins`/`pitch` are `None` for
/// families whose template does not encode them (CHIP/MELF have a fixed 2-pin,
/// no-pitch shape); `has_ep` is the `_1EP…` exposed-pad marker (QFN only).
#[derive(Debug, Clone)]
pub struct Ipc7351 {
    pub family: Family,
    pub pins: Option<u32>,
    /// Pitch in hundredths of a millimetre (e.g. `40` = 0.40mm).
    pub pitch_hundredths: Option<u32>,
    pub has_ep: bool,
}

/// A specific, name-able parse failure (→ E808 sub-cases).
#[derive(Debug)]
pub enum ParseErr {
    UnknownFamily,
    MissingDensity,
    Malformed(&'static str),
}

impl ParseErr {
    pub fn message(&self) -> String {
        match self {
            ParseErr::UnknownFamily => {
                "unrecognized IPC-7351 family prefix (expected one of QFP, QFN, \
                 SOIC, SOP, SOT, BGA, CHIP, MELF)"
                    .to_string()
            }
            ParseErr::MissingDensity => {
                "missing or invalid density suffix (expected one of N, L, M)".to_string()
            }
            ParseErr::Malformed(what) => format!("malformed IPC-7351 name: {}", what),
        }
    }
}

/// Cursor over the name's bytes (all ASCII in a valid name).
struct Cur<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Cur<'a> {
    fn new(s: &'a str) -> Self {
        Cur {
            b: s.as_bytes(),
            i: 0,
        }
    }
    fn eat(&mut self, lit: &str) -> bool {
        if self.b[self.i..].starts_with(lit.as_bytes()) {
            self.i += lit.len();
            true
        } else {
            false
        }
    }
    /// A run of ASCII digits as a u32 (`None` if no digit here, or overflow).
    fn uint(&mut self) -> Option<u32> {
        let start = self.i;
        while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
            self.i += 1;
        }
        if self.i == start {
            return None;
        }
        std::str::from_utf8(&self.b[start..self.i])
            .ok()?
            .parse()
            .ok()
    }
    /// A density level `N`/`L`/`M` at the end of the string.
    fn density_at_end(&mut self) -> Result<(), ParseErr> {
        if self.i + 1 == self.b.len() && matches!(self.b[self.i], b'N' | b'L' | b'M') {
            self.i += 1;
            Ok(())
        } else {
            Err(ParseErr::MissingDensity)
        }
    }
    fn at_end(&self) -> bool {
        self.i == self.b.len()
    }
}

pub fn parse(s: &str) -> Result<Ipc7351, ParseErr> {
    if !s.is_ascii() {
        return Err(ParseErr::Malformed("contains non-ASCII characters"));
    }
    let mut c = Cur::new(s);
    // Longest-prefix first so SOIC is not read as SO… + T, etc.
    if c.eat("QFP") {
        parse_qfp(&mut c)
    } else if c.eat("QFN") {
        parse_qfn(&mut c)
    } else if c.eat("SOIC") {
        parse_soic(&mut c, Family::Soic)
    } else if c.eat("SOP") {
        parse_soic(&mut c, Family::Sop)
    } else if c.eat("SOT") {
        parse_sot(&mut c)
    } else if c.eat("BGA") {
        parse_bga(&mut c)
    } else if c.eat("CHIP") {
        parse_chip(&mut c, Family::Chip)
    } else if c.eat("MELF") {
        parse_chip(&mut c, Family::Melf)
    } else {
        Err(ParseErr::UnknownFamily)
    }
}

fn dim(c: &mut Cur, what: &'static str) -> Result<u32, ParseErr> {
    c.uint().ok_or(ParseErr::Malformed(what))
}
fn lit(c: &mut Cur, l: &str, what: &'static str) -> Result<(), ParseErr> {
    if c.eat(l) {
        Ok(())
    } else {
        Err(ParseErr::Malformed(what))
    }
}

// `QFP{pitch}P{lsX}X{lsY}X{h}_{pins}{D}`  (`_` is IPC-7351's `-`)
fn parse_qfp(c: &mut Cur) -> Result<Ipc7351, ParseErr> {
    let pitch = dim(c, "expected the pitch after `QFP`")?;
    lit(c, "P", "expected `P` after the pitch")?;
    dim(c, "expected lead-span X")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected lead-span Y")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected height")?;
    lit(c, "_", "expected `_` (IPC-7351 `-`) before the pin count")?;
    let pins = dim(c, "expected the pin count")?;
    c.density_at_end()?;
    Ok(Ipc7351 {
        family: Family::Qfp,
        pins: Some(pins),
        pitch_hundredths: Some(pitch),
        has_ep: false,
    })
}

// `QFN{pins}{D}{pitch}P{bX}X{bY}[_1EP{eX}X{eY}]`  (`_` is IPC-7351's `-`)
fn parse_qfn(c: &mut Cur) -> Result<Ipc7351, ParseErr> {
    let pins = dim(c, "expected the pin count after `QFN`")?;
    // density is INTERIOR here (before the pitch) — read exactly one letter.
    if !(c.eat("N") || c.eat("L") || c.eat("M")) {
        return Err(ParseErr::MissingDensity);
    }
    let pitch = dim(c, "expected the pitch")?;
    lit(c, "P", "expected `P` after the pitch")?;
    dim(c, "expected body X")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected body Y")?;
    let has_ep = c.eat("_1EP");
    if has_ep {
        dim(c, "expected exposed-pad X after `_1EP`")?;
        lit(c, "X", "expected `X` in the exposed-pad size")?;
        dim(c, "expected exposed-pad Y")?;
    }
    if !c.at_end() {
        return Err(ParseErr::Malformed("trailing characters after the name"));
    }
    Ok(Ipc7351 {
        family: Family::Qfn,
        pins: Some(pins),
        pitch_hundredths: Some(pitch),
        has_ep,
    })
}

// `SOIC{pins}P{pitch}X{ls}X{h}{D}`  /  `SOP{pins}P{pitch}X{ls}X{h}{D}`
fn parse_soic(c: &mut Cur, fam: Family) -> Result<Ipc7351, ParseErr> {
    let pins = dim(c, "expected the pin count")?;
    lit(c, "P", "expected `P` after the pin count")?;
    let pitch = dim(c, "expected the pitch")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected lead span")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected height")?;
    c.density_at_end()?;
    Ok(Ipc7351 {
        family: fam,
        pins: Some(pins),
        pitch_hundredths: Some(pitch),
        has_ep: false,
    })
}

// `SOT{pins}P{pitch}X{bX}X{bY}{D}`
fn parse_sot(c: &mut Cur) -> Result<Ipc7351, ParseErr> {
    let pins = dim(c, "expected the pin count after `SOT`")?;
    lit(c, "P", "expected `P` after the pin count")?;
    let pitch = dim(c, "expected the pitch")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected body X")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected body Y")?;
    c.density_at_end()?;
    Ok(Ipc7351 {
        family: Family::Sot,
        pins: Some(pins),
        pitch_hundredths: Some(pitch),
        has_ep: false,
    })
}

// `BGA{pins}{C|N}{pitch}P{cols}X{rows}_{bX}X{bY}X{h}{D}`
fn parse_bga(c: &mut Cur) -> Result<Ipc7351, ParseErr> {
    let pins = dim(c, "expected the pin/ball count after `BGA`")?;
    if !(c.eat("C") || c.eat("N")) {
        return Err(ParseErr::Malformed(
            "expected `C` or `N` (collapsing) after the ball count",
        ));
    }
    let pitch = dim(c, "expected the pitch")?;
    lit(c, "P", "expected `P` after the pitch")?;
    dim(c, "expected columns")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected rows")?;
    lit(c, "_", "expected `_` before the body size")?;
    dim(c, "expected body X")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected body Y")?;
    lit(c, "X", "expected `X`")?;
    dim(c, "expected height")?;
    c.density_at_end()?;
    Ok(Ipc7351 {
        family: Family::Bga,
        pins: Some(pins),
        pitch_hundredths: Some(pitch),
        has_ep: false,
    })
}

// `CHIP_{EIA}`  /  `MELF_{EIA}` — a two-terminal passive; no density suffix.
fn parse_chip(c: &mut Cur, fam: Family) -> Result<Ipc7351, ParseErr> {
    lit(
        c,
        "_",
        "expected `_` (IPC-7351 `-`) after the family prefix",
    )?;
    dim(c, "expected an EIA size code (e.g. 0402)")?;
    if !c.at_end() {
        return Err(ParseErr::Malformed(
            "trailing characters after the EIA size",
        ));
    }
    Ok(Ipc7351 {
        family: fam,
        // A CHIP/MELF is a two-terminal part by definition; the name carries no
        // explicit pin/pitch, so the geometry check verifies the 2-pad shape.
        pins: Some(2),
        pitch_hundredths: None,
        has_ep: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_rfc_examples() {
        let qfn = parse("QFN60N40P700X700_1EP340X340").unwrap();
        assert_eq!(qfn.family, Family::Qfn);
        assert_eq!(qfn.pins, Some(60));
        assert_eq!(qfn.pitch_hundredths, Some(40));
        assert!(qfn.has_ep);

        let qfp = parse("QFP50P900X900X160_48N").unwrap();
        assert_eq!(qfp.family, Family::Qfp);
        assert_eq!(qfp.pins, Some(48));
        assert_eq!(qfp.pitch_hundredths, Some(50));
        assert!(!qfp.has_ep);

        let chip = parse("CHIP_0402").unwrap();
        assert_eq!(chip.family, Family::Chip);
        assert_eq!(chip.pins, Some(2));
    }

    #[test]
    fn qfn_without_ep() {
        let q = parse("QFN10N40P300X300").unwrap();
        assert!(!q.has_ep);
        assert_eq!(q.pins, Some(10));
    }

    #[test]
    fn sot_and_soic() {
        assert_eq!(
            parse("SOT5P95X290X160N").unwrap().pitch_hundredths,
            Some(95)
        );
        assert_eq!(parse("SOIC8P127X600X175N").unwrap().pins, Some(8));
    }

    #[test]
    fn unknown_family_is_free_form() {
        // A name outside the closed set is not an error here — the caller leaves
        // it as an ordinary RFC-016 identifier, unchecked.
        assert!(matches!(
            parse("FP_Crystal_SMD_3225"),
            Err(ParseErr::UnknownFamily)
        ));
        assert!(matches!(
            parse("WHAT10N40P300X300"),
            Err(ParseErr::UnknownFamily)
        ));
    }

    #[test]
    fn rejects_malformed() {
        assert!(matches!(
            parse("QFN10P300X300"),
            Err(ParseErr::MissingDensity)
        )); // no density
        assert!(matches!(
            parse("QFP50P900X900X160_48"),
            Err(ParseErr::MissingDensity)
        )); // no density
        assert!(matches!(
            parse("QFN10N40P300X300junk"),
            Err(ParseErr::Malformed(_))
        ));
        assert!(matches!(parse("CHIP0402"), Err(ParseErr::Malformed(_)))); // missing `_`
    }
}
