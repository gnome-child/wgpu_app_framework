#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Unrestricted,
    SignedInteger,
    UnsignedInteger,
    Decimal,
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

    pub fn decimal() -> Self {
        Self {
            kind: Kind::Decimal,
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
            Kind::Decimal => decimal_candidate(normalized),
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

fn decimal_candidate(candidate: &str) -> bool {
    let unsigned = candidate
        .strip_prefix('-')
        .or_else(|| candidate.strip_prefix('+'))
        .unwrap_or(candidate);
    if unsigned.is_empty() {
        return true;
    }
    let mut parts = unsigned.split(['e', 'E']);
    let mantissa = parts.next().unwrap_or_default();
    let exponent = parts.next();
    if parts.next().is_some() {
        return false;
    }
    let mut decimal_points = 0;
    let mantissa_ok = mantissa.chars().all(|character| {
        if character == '.' {
            decimal_points += 1;
            decimal_points <= 1
        } else {
            character.is_ascii_digit()
        }
    });
    if !mantissa_ok {
        return false;
    }
    let Some(exponent) = exponent else {
        return true;
    };
    if !mantissa.chars().any(|character| character.is_ascii_digit()) {
        return false;
    }
    exponent
        .strip_prefix('-')
        .or_else(|| exponent.strip_prefix('+'))
        .unwrap_or(exponent)
        .chars()
        .all(|character| character.is_ascii_digit())
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

    #[test]
    fn decimal_policy_accepts_float_intermediates_and_exponents() {
        for candidate in ["", "-", ".", "-.", "0.25", "+4.5", "1e", "1e-", "1E+3"] {
            assert!(
                matches!(Input::decimal().evaluate(candidate), Decision::Accept),
                "{candidate:?} is a lawful decimal draft"
            );
        }
        for candidate in ["--1", "1.2.3", "e3", "1ee2", "1e-+2", "NaN"] {
            assert!(
                matches!(Input::decimal().evaluate(candidate), Decision::Reject),
                "{candidate:?} is not a decimal draft"
            );
        }
    }
}
