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
pub(crate) struct Projection {
    table: interaction::Id,
    viewport_rect: Rect,
    surface_rect: Rect,
    columns: Vec<ResolvedColumn>,
}

#[derive(Clone)]
struct ResolvedColumn {
    identity: crate::table::HeaderCell,
    origin: i32,
    width: i32,
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
    fn column(
        header: &Frame,
        table: &Frame,
        projection: &Projection,
        identity: crate::table::HeaderCell,
    ) -> Option<Self> {
        let resolved = projection.column(identity)?;
        let boundary = projection
            .surface_rect
            .x()
            .saturating_add(resolved.origin)
            .saturating_add(resolved.width);
        debug_assert_eq!(header.rect().right(), boundary);
        Some(Self {
            axis: Axis::Column,
            boundary,
            rule_rect: Rect::new(
                boundary.saturating_sub(1),
                table.rect().y(),
                2,
                table.rect().height(),
            ),
            clip: header.clip(),
            floating_layer: table.is_floating_layer(),
            table_node: table.node_id(),
            column: Some(Column {
                identity,
                header_node: header.node_id(),
                header_rect: header.rect(),
                table_rect: projection.viewport_rect,
            }),
        })
    }

    fn row(row: &Frame, table: &Frame, projection: &Projection, boundary: i32) -> Self {
        Self {
            axis: Axis::Row,
            boundary,
            rule_rect: Rect::new(
                projection.surface_rect.x(),
                boundary.saturating_sub(1),
                projection.surface_rect.width(),
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
        let centered = self.boundary.saturating_sub(width / 2);
        let x = if self.boundary >= column.table_rect.x()
            && self.boundary <= column.table_rect.right()
        {
            centered.clamp(
                column.table_rect.x(),
                column.table_rect.right().saturating_sub(width),
            )
        } else {
            centered
        };
        Some(Rect::new(
            x,
            column.header_rect.y(),
            width,
            column.header_rect.height(),
        ))
    }
}

impl Projection {
    pub(crate) fn new(
        table: interaction::Id,
        viewport_rect: Rect,
        surface_rect: Rect,
        columns: impl IntoIterator<Item = (crate::table::HeaderCell, Rect)>,
    ) -> Self {
        Self {
            table,
            viewport_rect,
            surface_rect,
            columns: columns
                .into_iter()
                .map(|(identity, rect)| ResolvedColumn {
                    identity,
                    origin: rect.x(),
                    width: rect.width(),
                })
                .collect(),
        }
    }

    pub(crate) fn table(&self) -> interaction::Id {
        self.table
    }

    pub(crate) fn content_width(&self) -> i32 {
        self.surface_rect.width()
    }

    pub(crate) fn column_width(&self, column: interaction::Id) -> Option<i32> {
        self.columns
            .iter()
            .find(|resolved| resolved.identity.column() == column)
            .map(|resolved| resolved.width)
    }

    pub(crate) fn cell_rect(&self, column: interaction::Id, row_rect: Rect) -> Option<Rect> {
        let resolved = self
            .columns
            .iter()
            .find(|resolved| resolved.identity.column() == column)?;
        Some(Rect::new(
            self.surface_rect.x().saturating_add(resolved.origin),
            row_rect.y(),
            resolved.width,
            row_rect.height(),
        ))
    }

    fn column(&self, identity: crate::table::HeaderCell) -> Option<&ResolvedColumn> {
        self.columns
            .iter()
            .find(|resolved| resolved.identity == identity)
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
            let Some(projection) = owner_projection(frame, frames) else {
                continue;
            };
            let Some(track) = Track::column(frame, table, projection, identity) else {
                continue;
            };
            tracks.push(track);
            if !header_rows.contains(&table.node_id()) {
                tracks.push(Track::row(frame, table, projection, frame.rect().bottom()));
                header_rows.push(table.node_id());
            }
        } else if frame.table_row().is_some() {
            let Some(table) = owner_table(frame, frames) else {
                continue;
            };
            let Some(projection) = owner_projection(frame, frames) else {
                continue;
            };
            tracks.push(Track::row(frame, table, projection, frame.rect().bottom()));
        }
    }

    tracks
}

fn owner_projection<'a>(frame: &Frame, frames: &'a [Frame]) -> Option<&'a Projection> {
    frames
        .iter()
        .filter(|candidate| frame.is_descendant_of(candidate))
        .filter_map(Frame::table_projection)
        .last()
}

fn owner_table<'a>(frame: &Frame, frames: &'a [Frame]) -> Option<&'a Frame> {
    frames
        .iter()
        .filter(|candidate| {
            candidate.role() == view::Role::Table && frame.is_descendant_of(candidate)
        })
        .last()
}
