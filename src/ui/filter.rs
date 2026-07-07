use crate::paint;

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    ops: Vec<paint::FilterOp>,
}

impl Filter {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn blur(amount: f32) -> Self {
        Self::new().with_blur(amount)
    }

    pub fn liquid(params: paint::LiquidFilter) -> Self {
        Self::new().with_liquid(params)
    }

    pub fn stack(ops: impl IntoIterator<Item = paint::FilterOp>) -> Self {
        Self {
            ops: ops.into_iter().map(paint::FilterOp::clamped).collect(),
        }
    }

    pub fn with_blur(mut self, amount: f32) -> Self {
        self.ops.push(paint::FilterOp::blur(amount));
        self
    }

    pub fn with_liquid(mut self, params: paint::LiquidFilter) -> Self {
        self.ops.push(paint::FilterOp::liquid(params));
        self
    }

    pub fn ops(&self) -> &[paint::FilterOp] {
        &self.ops
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_stores_ordered_ops() {
        let filter = Filter::new()
            .with_blur(0.5)
            .with_liquid(paint::LiquidFilter {
                depth: 0.2,
                splay: 2.0,
                feather: 18.0,
                curve: 2.0,
            });

        assert_eq!(
            filter.ops(),
            &[
                paint::FilterOp::Blur { amount: 0.5 },
                paint::FilterOp::Liquid {
                    depth: 0.2,
                    splay: 2.0,
                    feather: 18.0,
                    curve: 2.0,
                },
            ]
        );
    }

    #[test]
    fn filter_clamps_stacked_ops() {
        let filter = Filter::stack([
            paint::FilterOp::Blur { amount: 2.0 },
            paint::FilterOp::Liquid {
                depth: -1.0,
                splay: -2.0,
                feather: -4.0,
                curve: 0.0,
            },
        ]);

        assert_eq!(
            filter.ops(),
            &[
                paint::FilterOp::Blur { amount: 1.0 },
                paint::FilterOp::Liquid {
                    depth: 0.0,
                    splay: 0.0,
                    feather: 0.0,
                    curve: 0.1,
                },
            ]
        );
    }
}
