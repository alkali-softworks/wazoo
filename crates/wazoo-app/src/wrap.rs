/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Wrap Widget Component
 *
 * A layout widget that arranges child elements horizontally and wraps them onto
 * new lines when the horizontal constraint is reached. Supports item alignment,
 * horizontal and vertical spacing, and an optional `fill_last` mode for tag-style inputs.
 */

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::mouse;
use iced::{Alignment, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector};

/// A container that distributes its elements horizontally and wraps onto new lines.
pub struct Wrap<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    width: Length,
    height: Length,
    spacing: f32,
    vertical_spacing: f32,
    align_items: Alignment,
    fill_last: bool,
    min_fill_width: f32,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Wrap<'a, Message, Theme, Renderer> {
    /// Creates an empty [`Wrap`] layout.
    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Shrink,
            spacing: 0.0,
            vertical_spacing: 0.0,
            align_items: Alignment::Start,
            fill_last: false,
            min_fill_width: 70.0,
            children: Vec::new(),
        }
    }

    /// Creates a [`Wrap`] layout initialized with the given elements.
    pub fn with_elements(
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            children: children.into_iter().collect(),
            ..Self::new()
        }
    }

    /// Sets the horizontal spacing between items in each row.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the vertical spacing between wrapped rows.
    pub fn vertical_spacing(mut self, vertical_spacing: impl Into<Pixels>) -> Self {
        self.vertical_spacing = vertical_spacing.into().0;
        self
    }

    /// Sets the vertical alignment of elements within each wrapped line.
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Sets the width of the [`Wrap`] container.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Wrap`] container.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Enables flexible filling for the last child element.
    ///
    /// If there is at least `min_fill_width` remaining on the current line, the last child
    /// expands to fill the remainder of that line. Otherwise, it wraps to a new line and
    /// expands to full width.
    pub fn fill_last(mut self, min_fill_width: f32) -> Self {
        self.fill_last = true;
        self.min_fill_width = min_fill_width;
        self
    }

    /// Appends an element to the [`Wrap`] layout.
    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message, Theme, Renderer> Default for Wrap<'a, Message, Theme, Renderer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Wrap<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let max_width = limits.max().width;

        if self.children.is_empty() {
            return layout::Node::new(limits.resolve(self.width, self.height, Size::ZERO));
        }

        let child_limits = limits.loose();
        let spacing = self.spacing;
        let vertical_spacing = self.vertical_spacing;

        let mut children_nodes = Vec::with_capacity(self.children.len());
        let mut intrinsic_width: f32 = 0.0;
        let mut row_start = 0;
        let mut row_height: f32 = 0.0;
        let mut x = 0.0;
        let mut y = 0.0;

        let align_factor = match self.align_items {
            Alignment::Start => 0.0,
            Alignment::Center => 2.0,
            Alignment::End => 1.0,
        };

        let align_y_for_row = |row_start: std::ops::Range<usize>,
                               row_height: f32,
                               children: &mut Vec<layout::Node>| {
            if align_factor != 0.0 {
                for node in &mut children[row_start] {
                    let height = node.size().height;
                    node.translate_mut(Vector::new(
                        0.0,
                        (row_height - height) / align_factor,
                    ));
                }
            }
        };

        let num_children = self.children.len();

        for (i, child) in self.children.iter_mut().enumerate() {
            let is_last_and_fill = self.fill_last && (i == num_children - 1);

            let node = if is_last_and_fill {
                let remaining_width = (max_width - x).max(0.0);
                if x > 0.0 && remaining_width < self.min_fill_width {
                    // Wrap to next line before laying out the last child
                    intrinsic_width = intrinsic_width.max(x - spacing);
                    align_y_for_row(row_start..i, row_height, &mut children_nodes);
                    y += row_height + vertical_spacing;
                    x = 0.0;
                    row_start = i;
                    row_height = 0.0;

                    let last_limits = layout::Limits::new(
                        Size::new(max_width, 0.0),
                        Size::new(max_width, limits.max().height),
                    );
                    child.as_widget_mut().layout(&mut tree.children[i], renderer, &last_limits)
                } else {
                    let target_width = if x > 0.0 { remaining_width } else { max_width };
                    let last_limits = layout::Limits::new(
                        Size::new(target_width, 0.0),
                        Size::new(target_width, limits.max().height),
                    );
                    child.as_widget_mut().layout(&mut tree.children[i], renderer, &last_limits)
                }
            } else {
                let node = child.as_widget_mut().layout(
                    &mut tree.children[i],
                    renderer,
                    &child_limits,
                );
                let child_size = node.size();

                if x > 0.0 && x + child_size.width > max_width {
                    intrinsic_width = intrinsic_width.max(x - spacing);
                    align_y_for_row(row_start..i, row_height, &mut children_nodes);
                    y += row_height + vertical_spacing;
                    x = 0.0;
                    row_start = i;
                    row_height = 0.0;
                }
                node
            };

            let child_size = node.size();
            row_height = row_height.max(child_size.height);
            let mut positioned_node = node;
            positioned_node.move_to_mut(Point::new(x, y));
            children_nodes.push(positioned_node);

            x += child_size.width + spacing;
        }

        if x > 0.0 {
            intrinsic_width = intrinsic_width.max(x - spacing);
        }

        align_y_for_row(row_start..children_nodes.len(), row_height, &mut children_nodes);

        let total_height = y + row_height;
        let resolved_size = limits.resolve(
            self.width,
            self.height,
            Size::new(intrinsic_width, total_height),
        );

        layout::Node::with_children(resolved_size, children_nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), child_layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, child_layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), child_layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), child_layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, child_layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            for ((child, tree), child_layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, child_layout)| child_layout.bounds().intersects(&clipped_viewport))
            {
                child.as_widget().draw(
                    tree,
                    renderer,
                    theme,
                    style,
                    child_layout,
                    cursor,
                    &clipped_viewport,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Wrap<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(wrap: Wrap<'a, Message, Theme, Renderer>) -> Self {
        Element::new(wrap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text;

    #[test]
    fn test_wrap_builder_defaults_and_options() {
        let wrap: Wrap<'static, ()> = Wrap::new()
            .spacing(10)
            .vertical_spacing(12)
            .align_items(Alignment::Center)
            .fill_last(85.0);

        assert_eq!(wrap.spacing, 10.0);
        assert_eq!(wrap.vertical_spacing, 12.0);
        assert_eq!(wrap.align_items, Alignment::Center);
        assert!(wrap.fill_last);
        assert_eq!(wrap.min_fill_width, 85.0);
        assert_eq!(wrap.children.len(), 0);
    }

    #[test]
    fn test_wrap_with_elements_and_tree() {
        let wrap: Wrap<'static, ()> = Wrap::with_elements(vec![
            text("tag 1").into(),
            text("tag 2").into(),
            text("tag 3").into(),
        ]);

        assert_eq!(wrap.children.len(), 3);
        let tree = Tree::new(&wrap as &dyn Widget<(), iced::Theme, iced::Renderer>);
        assert_eq!(tree.children.len(), 3);
    }
}
