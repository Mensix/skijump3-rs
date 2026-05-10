use crate::ui::widget::WidgetId;

struct WidgetEntry {
    widget: Box<dyn crate::ui::widget::Widget>,
    dirty: bool,
}

pub struct WidgetTree {
    entries: Vec<WidgetEntry>,
    root: Option<WidgetId>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self { entries: Vec::new(), root: None }
    }

    pub fn mount(&mut self, widget: Box<dyn crate::ui::widget::Widget>) -> WidgetId {
        let id = WidgetId(self.entries.len() as u32);
        self.entries.push(WidgetEntry { widget, dirty: true });
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    pub fn mark_dirty(&mut self, id: WidgetId) {
        if let Some(e) = self.entries.get_mut(id.0 as usize) {
            e.dirty = true;
        }
    }

    pub fn mark_dirty_all(&mut self) {
        for e in &mut self.entries {
            e.dirty = true;
        }
    }

    pub fn mark_clean(&mut self, id: WidgetId) {
        if let Some(e) = self.entries.get_mut(id.0 as usize) {
            e.dirty = false;
        }
    }

    pub fn mark_clean_all(&mut self) {
        for e in &mut self.entries {
            e.dirty = false;
        }
    }

    pub fn render(&mut self, ctx: &mut crate::ui::paint::PaintCtx) {
        for entry in &mut self.entries {
            if entry.dirty {
                entry.widget.paint(ctx, WidgetId::INVALID);
                entry.dirty = false;
            }
        }
    }

    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}