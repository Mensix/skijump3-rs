use crate::ui::paint::PaintCtx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u32);

impl WidgetId {
    pub const INVALID: Self = Self(u32::MAX);
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::INVALID
    }
}

pub trait Props: 'static + Send + Sync {}

pub trait Widget: Send + Sync {
    fn paint(&self, ctx: &mut PaintCtx, id: WidgetId);
}