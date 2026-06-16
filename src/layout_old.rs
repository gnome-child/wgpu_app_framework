use crate::geometry::{Rect, area, point};
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Fit,
    Fill,
    Fixed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min: area::Logical,
    pub max: area::Logical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Box {
    id: ui::Id,
    path: ui::Path,
    rect: Rect,
    children: Vec<Box>,
}

impl Constraints {
    pub fn new(min: area::Logical, max: area::Logical) -> Self {
        Self { min, max }
    }

    pub fn loose(max: area::Logical) -> Self {
        Self {
            min: area::logical(0.0, 0.0),
            max,
        }
    }

    pub fn tight(area: area::Logical) -> Self {
        Self {
            min: area,
            max: area,
        }
    }

    pub fn min(self) -> area::Logical {
        self.min
    }

    pub fn max(self) -> area::Logical {
        self.max
    }
}

impl Insets {
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    pub const fn splat(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

impl Box {
    pub fn new(id: ui::Id, rect: Rect, children: Vec<Box>) -> Self {
        Self::with_path(ui::Path::root(id), rect, children)
    }

    pub fn with_path(path: ui::Path, rect: Rect, children: Vec<Box>) -> Self {
        let id = path
            .leaf()
            .expect("layout boxes must have at least one path segment");

        Self {
            id,
            path,
            rect,
            children,
        }
    }

    pub fn hit_test(&self, position: point::Logical) -> Option<ui::Path> {
        self.hit_test_where(position, |_| true)
    }

    pub fn id(&self) -> ui::Id {
        self.id
    }

    pub fn path(&self) -> &ui::Path {
        &self.path
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn children(&self) -> &[Box] {
        &self.children
    }

    pub fn with_children(mut self, children: Vec<Box>) -> Self {
        self.children = children;
        self
    }

    pub fn hit_test_where(
        &self,
        position: point::Logical,
        accepts: impl Copy + Fn(&ui::Path) -> bool,
    ) -> Option<ui::Path> {
        if !contains(self.rect, position) {
            return None;
        }

        for child in self.children.iter().rev() {
            if let Some(id) = child.hit_test_where(position, accepts) {
                return Some(id);
            }
        }

        accepts(&self.path).then_some(self.path.clone())
    }

    pub fn find(&self, id: ui::Id) -> Option<&Box> {
        if self.id == id {
            return Some(self);
        }

        self.children.iter().find_map(|child| child.find(id))
    }

    pub fn find_path(&self, path: &ui::Path) -> Option<&Box> {
        if &self.path == path {
            return Some(self);
        }

        self.children.iter().find_map(|child| child.find_path(path))
    }
}

fn contains(rect: Rect, position: point::Logical) -> bool {
    let x = position.x();
    let y = position.y();
    let left = rect.origin.x();
    let top = rect.origin.y();
    let right = left + rect.area.width();
    let bottom = top + rect.area.height();

    x >= left && x < right && y >= top && y < bottom
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");

    #[test]
    fn hit_testing_returns_deepest_matching_box() {
        let layout = Box::new(
            ROOT,
            Rect::new(point::logical(0.0, 0.0), area::logical(100.0, 100.0)),
            vec![Box::new(
                CHILD,
                Rect::new(point::logical(10.0, 10.0), area::logical(20.0, 20.0)),
                Vec::new(),
            )],
        );

        assert_eq!(
            layout.hit_test(point::logical(15.0, 15.0)),
            Some(ui::Path::from(CHILD))
        );
        assert_eq!(
            layout.hit_test(point::logical(90.0, 90.0)),
            Some(ui::Path::from(ROOT))
        );
        assert_eq!(layout.hit_test(point::logical(110.0, 90.0)), None);
    }
}
