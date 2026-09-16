//! Checking a person's signed record without the forge: the signature a
//! receipt carries, over the figures their page shows.

use crate::receipt::verify_signed;
use serde_json::Value;

/// What a verified record says.
pub struct Summary {
    /// The forge that issued it, when the record names one.
    pub forge: Option<String>,
    pub principal: String,
    pub display: String,
    pub kind: String,
    pub window_days: u64,
    pub issued_at: String,
    pub landed: u64,
    pub abandoned: u64,
    pub claims: u64,
    pub judged: u64,
    pub reproduced: u64,
    pub disputed: u64,
    pub blocks: u64,
    pub audits: u64,
    pub key: String,
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind.as_str() {
            "human" => "person",
            "team" => "organisation",
            other => other,
        };
        writeln!(
            f,
            "verified  the record of {} (@{}), {kind}",
            self.display, self.principal
        )?;
        if let Some(forge) = &self.forge {
            writeln!(f, "  forge     {forge}")?;
        }
        writeln!(
            f,
            "  window    {} days, issued {}",
            self.window_days,
            self.issued_at.get(..10).unwrap_or(&self.issued_at)
        )?;
        writeln!(
            f,
            "  landed    {}, {} abandoned",
            self.landed, self.abandoned
        )?;
        writeln!(
            f,
            "  claims    {} made, {} of {} re-run reproduced, {} disputed",
            self.claims, self.reproduced, self.judged, self.disputed
        )?;
        writeln!(f, "  blocked   {} of {} looks", self.blocks, self.audits)?;
        write!(f, "  key       {}", self.key)
    }
}

/// Verify a signed record document. `expected_key` is a fingerprint or a
/// base64 public key it must have been signed with.
pub fn verify(document: &str, expected_key: Option<&str>) -> anyhow::Result<Summary> {
    let (body, key) = verify_signed(document, "record", expected_key)?;
    let num = |k: &str| body.get(k).and_then(Value::as_u64).unwrap_or(0);
    let text = |k: &str| body.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
    Ok(Summary {
        forge: body.get("forge").and_then(Value::as_str).map(str::to_owned),
        principal: text("principal"),
        display: text("display"),
        kind: text("kind"),
        window_days: num("window_days"),
        issued_at: text("issued_at"),
        landed: num("landed"),
        abandoned: num("abandoned"),
        claims: num("claims"),
        judged: num("judged"),
        reproduced: num("reproduced"),
        disputed: num("disputed"),
        blocks: num("blocks"),
        audits: num("audits"),
        key,
    })
}
