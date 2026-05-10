use crate::ui::paint::PaintCtx;
use crate::ui::widget::{Widget, Props, WidgetId};

#[derive(Debug, Clone)]
pub struct ContainerProps;

impl Props for ContainerProps {}

pub struct Container {
    pub children: Vec<Box<dyn Widget>>,
}

impl Container {
    pub fn new() -> Self {
        Self { children: Vec::new() }
    }

    pub fn with_child(mut self, widget: Box<dyn Widget>) -> Self {
        self.children.push(widget);
        self
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Container {
    fn paint(&self, ctx: &mut PaintCtx, _id: WidgetId) {
        for child in &self.children {
            child.paint(ctx, WidgetId::INVALID);
        }
    }
}