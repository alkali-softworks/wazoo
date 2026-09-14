/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Infinite Scroll Stream Layout
 *
 * Implements a custom Iced widget that arranges child video players into a smooth,
 * continuously scrolling vertical stream with precise pixel positioning.
 */

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::mouse;
use iced::{Element, Event, Length, Point, Rectangle, Size};

/// A custom vertical streaming widget for Scroll Mode ("The Infinity Stream").
/// Lays out children at arbitrary Y positions and clips rendered output to the container viewport.
pub struct ScrollStream<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    width: Length,
    height: Length,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    positions: Vec<(f32, f32)>, // (y_pos, height)
}

impl<'a, Message, Theme, Renderer> ScrollStream<'a, Message, Theme, Renderer> {
    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            children: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn push(
        mut self,
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
        y_pos: f32,
        height: f32,
    ) -> Self {
        self.children.push(child.into());
        self.positions.push((y_pos, height));
        self
    }
}

impl<'a, Message, Theme, Renderer> Default for ScrollStream<'a, Message, Theme, Renderer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ScrollStream<'a, Message, Theme, Renderer>
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
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        let mut nodes = Vec::with_capacity(self.children.len());

        for (i, (child, &(y_pos, item_h))) in
            self.children.iter_mut().zip(&self.positions).enumerate()
        {
            let child_limits =
                layout::Limits::new(Size::new(size.width, item_h), Size::new(size.width, item_h));

            let mut child_node =
                child
                    .as_widget_mut()
                    .layout(&mut tree.children[i], renderer, &child_limits);
            child_node.move_to_mut(Point::new(0.0, y_pos));
            nodes.push(child_node);
        }

        layout::Node::with_children(size, nodes)
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
        let bounds = layout.bounds();
        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            renderer.with_layer(clipped_viewport, |renderer| {
                for ((child, tree), child_layout) in self
                    .children
                    .iter()
                    .zip(&tree.children)
                    .zip(layout.children())
                {
                    // Draw only elements whose bounding box intersects the clipped viewport
                    if child_layout
                        .bounds()
                        .intersection(&clipped_viewport)
                        .is_some()
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
            });
        }
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
        let bounds = layout.bounds();
        let Some(clipped_viewport) = bounds.intersection(viewport) else {
            return;
        };

        for ((child, tree), child_layout) in self
            .children
            .iter_mut()
            .rev()
            .zip(tree.children.iter_mut().rev())
            .zip(layout.children().rev())
        {
            child.as_widget_mut().update(
                tree,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &clipped_viewport,
            );

            if shell.is_event_captured() {
                return;
            }
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
        let bounds = layout.bounds();
        let Some(clipped_viewport) = bounds.intersection(viewport) else {
            return mouse::Interaction::None;
        };

        self.children
            .iter()
            .rev()
            .zip(tree.children.iter().rev())
            .zip(layout.children().rev())
            .map(|((child, tree), child_layout)| {
                child.as_widget().mouse_interaction(
                    tree,
                    child_layout,
                    cursor,
                    &clipped_viewport,
                    renderer,
                )
            })
            .find(|&interaction| interaction != mouse::Interaction::None)
            .unwrap_or_default()
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
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

impl<'a, Message, Theme, Renderer> From<ScrollStream<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(stream: ScrollStream<'a, Message, Theme, Renderer>) -> Self {
        Element::new(stream)
    }
}
