//! The canonical, length-prefixed action binding (l2-tool-receipts.md §4.2)
//! — the MAC input. Every field is length-prefixed so the encoding is
//! injective: naive concatenation lets a boundary shift between two
//! adjacent fields produce byte-identical output for two different
//! actions (`kind="ab", inputs="c"` colliding with `kind="a",
//! inputs="bc"`), which would let a model forge a receipt for an action it
//! never took without ever touching the key.

/// Distinguishes receipt MACs from any other keyed hash this project may
/// later compute with a different key, so two subsystems can never accept
/// each other's tags.
const DOMAIN_TAG: &[u8] = b"cronus.tool_receipt.v1";

/// The fields bound into a receipt's MAC. `action_id` is a per-session
/// monotonic counter assigned by the dispatcher — the same identity the
/// ledger keys on — and is what makes a receipt witness *this invocation*
/// rather than merely an action shape: two calls identical in every other
/// field still bind to different tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBinding {
    pub action_id: u64,
    pub action_kind: String,
    pub inputs_digest: Vec<u8>,
    pub outcome_tag: String,
    pub result_digest: Vec<u8>,
    pub timestamp_ms: u64,
}

fn push_field(buf: &mut Vec<u8>, field: &[u8]) {
    buf.extend_from_slice(&(field.len() as u32).to_le_bytes());
    buf.extend_from_slice(field);
}

/// Serialize `binding` into the canonical MAC input: the domain tag, then
/// every field length-prefixed in a fixed order, so the byte stream is
/// injective — no two distinct `ActionBinding`s ever encode to the same
/// bytes.
pub fn encode(binding: &ActionBinding) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(DOMAIN_TAG);
    buf.extend_from_slice(&binding.action_id.to_le_bytes());
    push_field(&mut buf, binding.action_kind.as_bytes());
    push_field(&mut buf, &binding.inputs_digest);
    push_field(&mut buf, binding.outcome_tag.as_bytes());
    push_field(&mut buf, &binding.result_digest);
    buf.extend_from_slice(&binding.timestamp_ms.to_le_bytes());
    buf
}

/// Digest an arbitrary payload before it enters a binding — inputs and
/// results are bound as digests, never raw, so a secret-bearing argument
/// or a large payload never lands in the MAC input, and token cost stays
/// independent of payload size.
pub fn digest(data: &[u8]) -> Vec<u8> {
    blake3::hash(data).as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(action_kind: &str, inputs: &[u8]) -> ActionBinding {
        ActionBinding {
            action_id: 1,
            action_kind: action_kind.to_string(),
            inputs_digest: inputs.to_vec(),
            outcome_tag: "ok".to_string(),
            result_digest: b"result".to_vec(),
            timestamp_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn boundary_shifting_across_kind_and_inputs_yields_distinct_bytes() {
        // Naive concatenation would make these two collide: "ab" + "c" ==
        // "a" + "bc" byte-for-byte. Length-prefixing must keep them apart.
        let a = fixed("ab", b"c");
        let b = fixed("a", b"bc");
        assert_ne!(
            encode(&a),
            encode(&b),
            "length-prefixed fields must not collide under a boundary shift"
        );
    }

    #[test]
    fn every_field_carries_a_little_endian_length_prefix() {
        let binding = fixed("deploy", b"prod");
        let buf = encode(&binding);
        let kind_len_offset = DOMAIN_TAG.len() + 8; // domain_tag + u64_le(action_id)
        let expected_len = (binding.action_kind.len() as u32).to_le_bytes();
        assert_eq!(
            &buf[kind_len_offset..kind_len_offset + 4],
            &expected_len,
            "action_kind's length prefix must be little-endian and immediately precede it"
        );
    }

    #[test]
    fn the_domain_separation_tag_leads_the_buffer() {
        let buf = encode(&fixed("deploy", b"prod"));
        assert!(buf.starts_with(DOMAIN_TAG));
    }

    #[test]
    fn distinct_action_ids_produce_distinct_encodings_with_every_other_field_equal() {
        let mut a = fixed("deploy", b"prod");
        let mut b = a.clone();
        a.action_id = 1;
        b.action_id = 2;
        assert_ne!(encode(&a), encode(&b));
    }
}
