use super::super::{
    composition,
    geometry::{Point, Rect},
    interaction, view,
};
use super::frame::{Clip, Frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    Column,
    Row,
}

#[derive(Clone)]
pub(crate) struct Track {
    axis: Axis,
    boundary: i32,
    rule_rect: Rect,
    clip: Option<Clip>,
    floating_layer: bool,
    table_node: composition::NodeId,
    column: Option<Column>,
}

#[derive(Clone)]
struct Column {
    identity: crate::table::HeaderCell,
    header_node: composition::NodeId,
    header_rect: Rect,
    table_rect: Rect,
}

impl Track {
    fn column(header: &Frame, table: &Frame, identity: crate::table::HeaderCell) -> Self {
        let boundary = header.rect().right();
        Self {
            axis: Axis::Column,
            boundary,
            rule_rect: Rect::new(
                boundary.saturating_sub(1),
                table.rect().y(),
                2,
                table.rect().height(),
            ),
            clip: table.clip(),
            floating_layer: table.is_floating_layer(),
            table_node: table.node_id(),
            column: Some(Column {
                identity,
                header_node: header.node_id(),
                header_rect: header.rect(),
                table_rect: table.rect(),
            }),
        }
    }

    fn row(row: &Frame, table: &Frame, boundary: i32) -> Self {
        Self {
            axis: Axis::Row,
            boundary,
            rule_rect: Rect::new(
                table.rect().x(),
                boundary.saturating_sub(1),
                table.rect().width(),
                2,
            ),
            clip: row.clip(),
            floating_layer: table.is_floating_layer(),
            table_node: table.node_id(),
            column: None,
        }
    }

    pub(crate) fn axis(&self) -> Axis {
        self.axis
    }

    pub(crate) fn rule_rect(&self) -> Rect {
        self.rule_rect
    }

    #[cfg(test)]
    pub(crate) fn boundary(&self) -> i32 {
        self.boundary
    }

    pub(crate) fn clip(&self) -> Option<Clip> {
        self.clip
    }

    pub(crate) fn is_floating_layer(&self) -> bool {
        self.floating_layer
    }

    pub(crate) fn table_node(&self) -> composition::NodeId {
        self.table_node
    }

    pub(crate) fn header_node(&self) -> Option<composition::NodeId> {
        self.column.as_ref().map(|column| column.header_node)
    }

    #[cfg(test)]
    pub(crate) fn column_identity(&self) -> Option<crate::table::HeaderCell> {
        self.column.as_ref().map(|column| column.identity)
    }

    pub(crate) fn divider_target(&self) -> Option<interaction::Target> {
        self.column.as_ref().map(|column| {
            interaction::Target::table_divider_node(column.header_node, "Resize table column")
        })
    }

    pub(crate) fn resize_action_at(&self, point: Point) -> Option<view::Action> {
        let column = self.column.as_ref()?;
        let width = point
            .x()
            .saturating_sub(column.header_rect.x())
            .max(crate::table::COLUMN_MIN_WIDTH);
        Some(view::Action::resize_table_column(column.identity, width))
    }

    pub(crate) fn accepts_resize_hit(&self, point: Point) -> bool {
        self.divider_hit_rect()
            .is_some_and(|rect| rect.contains(point))
            && self.clip.is_none_or(|clip| clip.contains(point))
    }

    pub(crate) fn divider_hit_rect(&self) -> Option<Rect> {
        let column = self.column.as_ref()?;
        let width = crate::table::DIVIDER_HIT_WIDTH.min(column.table_rect.width());
        if width <= 0 {
            return None;
        }
        let x = self.boundary.saturating_sub(width / 2).clamp(
            column.table_rect.x(),
            column.table_rect.right().saturating_sub(width),
        );
        Some(Rect::new(
            x,
            column.header_rect.y(),
            width,
            column.header_rect.height(),
        ))
    }
}

/// Projects already-resolved table frames into grid tracks; it never allocates track sizes.
pub(crate) fn project(frames: &[Frame]) -> Vec<Track> {
    let mut tracks = Vec::new();
    let mut header_rows = Vec::new();

    for frame in frames {
        if let Some(identity) = frame.table_header_cell() {
            let Some(table) = owner_table(frame, frames) else {
                continue;
            };
            tracks.push(Track::column(frame, table, identity));
            if !header_rows.contains(&table.node_id()) {
                tracks.push(Track::row(frame, table, frame.rect().bottom()));
                header_rows.push(table.node_id());
            }
        } else if frame.table_row().is_some() {
            let Some(table) = owner_table(frame, frames) else {
                continue;
            };
            tracks.push(Track::row(frame, table, frame.rect().bottom()));
        }
    }

    tracks
}

fn owner_table<'a>(frame: &Frame, frames: &'a [Frame]) -> Option<&'a Frame> {
    frames
        .iter()
        .filter(|candidate| {
            candidate.role() == view::Role::Table && frame.is_descendant_of(candidate)
        })
        .last()
}
