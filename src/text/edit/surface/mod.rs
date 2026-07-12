mod area;
mod field;
mod mode;
mod projection;

pub use area::{Area, AreaWrap};
pub use field::{Field, Obscuring};
pub use mode::FieldMode;

pub(crate) use projection::{
    FieldProjection, PositionMap, PreeditProjection, projected_state_for_field,
};

use super::super::buffer::Buffer;
use super::{State, ViewState};

#[derive(Debug, Clone, PartialEq)]
pub enum Surface {
    Field(Field),
    Area(Area),
}

impl Surface {
    pub fn buffer(&self) -> &Buffer {
        match self {
            Self::Field(field) => field.buffer(),
            Self::Area(area) => area.buffer(),
        }
    }

    pub fn state(&self) -> State {
        match self {
            Self::Field(field) => field.state(),
            Self::Area(area) => area.state(),
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field(_))
    }

    pub fn is_area(&self) -> bool {
        matches!(self, Self::Area(_))
    }

    pub fn as_field(&self) -> Option<&Field> {
        match self {
            Self::Field(field) => Some(field),
            Self::Area(_) => None,
        }
    }

    pub fn as_area(&self) -> Option<&Area> {
        match self {
            Self::Field(_) => None,
            Self::Area(area) => Some(area),
        }
    }

    pub fn placeholder(&self) -> Option<&str> {
        match self {
            Self::Field(field) => field.placeholder(),
            Self::Area(area) => area.placeholder(),
        }
    }

    pub fn is_editable(&self) -> bool {
        match self {
            Self::Field(field) => field.is_editable(),
            Self::Area(area) => area.is_editable(),
        }
    }

    pub fn is_read_only(&self) -> bool {
        match self {
            Self::Field(field) => field.is_read_only(),
            Self::Area(area) => area.is_read_only(),
        }
    }

    pub fn is_disabled(&self) -> bool {
        match self {
            Self::Field(field) => field.is_disabled(),
            Self::Area(area) => area.is_disabled(),
        }
    }

    pub fn is_selectable(&self) -> bool {
        match self {
            Self::Field(field) => field.is_selectable(),
            Self::Area(area) => area.is_selectable(),
        }
    }

    pub fn accepts_text_input(&self) -> bool {
        match self {
            Self::Field(field) => field.accepts_text_input(),
            Self::Area(area) => area.accepts_text_input(),
        }
    }

    pub fn paints_caret(&self) -> bool {
        match self {
            Self::Field(field) => field.paints_caret(),
            Self::Area(area) => area.paints_caret(),
        }
    }

    pub fn allows_text_mutation(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_text_mutation(),
            Self::Area(area) => area.allows_text_mutation(),
        }
    }

    pub fn allows_copy(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_copy(),
            Self::Area(area) => area.allows_copy(),
        }
    }

    pub fn allows_cut(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_cut(),
            Self::Area(area) => area.allows_cut(),
        }
    }

    pub fn presentation_text(&self) -> String {
        match self {
            Self::Field(field) => field.presentation_text(),
            Self::Area(area) => area.presentation_text(),
        }
    }

    pub fn presentation_text_for_state(&self, state: &ViewState) -> String {
        match self {
            Self::Field(field) => field.presentation_text_for_state(state),
            Self::Area(area) => area.presentation_text_for_state(state),
        }
    }
}

impl From<Field> for Surface {
    fn from(value: Field) -> Self {
        Self::Field(value)
    }
}

impl From<Area> for Surface {
    fn from(value: Area) -> Self {
        Self::Area(value)
    }
}
