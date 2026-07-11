#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Unrestricted,
    SignedInteger,
    UnsignedInteger,
}

pub(crate) enum Decision {
    Accept,
    Normalize(String),
    Reject,
}

impl Input {
    pub fn unrestricted() -> Self {
        Self {
            kind: Kind::Unrestricted,
        }
    }

    pub fn signed_integer() -> Self {
        Self {
            kind: Kind::SignedInteger,
        }
    }

    pub fn unsigned_integer() -> Self {
        Self {
            kind: Kind::UnsignedInteger,
        }
    }

    pub(crate) fn evaluate(self, proposed: &str) -> Decision {
        if self.kind == Kind::Unrestricted {
            return Decision::Accept;
        }

        let normalized = proposed.trim();
        let accepted = match self.kind {
            Kind::Unrestricted => true,
            Kind::SignedInteger => {
                normalized.is_empty()
                    || normalized == "-"
                    || normalized
                        .strip_prefix('-')
                        .unwrap_or(normalized)
                        .chars()
                        .all(|character| character.is_ascii_digit())
            }
            Kind::UnsignedInteger => {
                normalized.is_empty()
                    || normalized
                        .chars()
                        .all(|character| character.is_ascii_digit())
            }
        };
        if !accepted {
            return Decision::Reject;
        }
        if normalized == proposed {
            Decision::Accept
        } else {
            Decision::Normalize(normalized.to_owned())
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::unrestricted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_policies_accept_intermediate_drafts_and_normalize_edges() {
        assert!(matches!(
            Input::signed_integer().evaluate(""),
            Decision::Accept
        ));
        assert!(matches!(
            Input::signed_integer().evaluate("-"),
            Decision::Accept
        ));
        assert!(matches!(
            Input::unsigned_integer().evaluate("-"),
            Decision::Reject
        ));
        assert!(matches!(
            Input::signed_integer().evaluate("-42"),
            Decision::Accept
        ));
        assert!(matches!(
            Input::unsigned_integer().evaluate("42"),
            Decision::Accept
        ));
        assert!(matches!(
            Input::signed_integer().evaluate(" -42 "),
            Decision::Normalize(value) if value == "-42"
        ));
        assert!(matches!(
            Input::signed_integer().evaluate("--4"),
            Decision::Reject
        ));
        assert!(matches!(
            Input::unsigned_integer().evaluate("4x"),
            Decision::Reject
        ));
    }
}
