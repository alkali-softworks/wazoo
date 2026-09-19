/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * SlideDown Transition Layout Widget
 *
 * A custom Iced container widget that animates its child sliding vertically
 * with hardware clipping. Supports full layout offset, smooth cubic ease-out,
 * and optional height cropping.
 */

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// A container widget that animates a child sliding down along the Y axis.
///
/// `progress` is a normalized float between `0.0` (fully retracted/hidden)
/// and `1.0` (fully extended/visible).
/// When `crop_height` is true, the outer layout bounds expand vertically from `0`
/// to the child's natural height, suitable for unfolding dropdowns.
/// When `crop_height` is false, the outer layout bounds retain the child's full height,
/// suitable for top overlays sliding in from offscreen.
pub struct SlideDown<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    progress: f32,
    crop_height: bool,
}

impl<'a, Message: 'a, Theme: 'a, Renderer: 'a> SlideDown<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        progress: f32,
        crop_height: bool,
    ) -> Self {
        Self {
            content: content.into(),
            progress: progress.clamp(0.0, 1.0),
            crop_height,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SlideDown<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut child_node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let child_size = child_node.size();

        let p = self.progress.clamp(0.0, 1.0);
        // Snappy cubic ease-out: 1 - (1 - p)^3
        let eased = 1.0 - (1.0 - p).powi(3);

        let offset_y = -child_size.height * (1.0 - eased);
        child_node.move_to_mut(Point::new(0.0, offset_y));

        let bounds_height = if self.crop_height {
            child_size.height * eased
        } else {
            child_size.height
        };

        layout::Node::with_children(
            Size::new(child_size.width, bounds_height),
            vec![child_node],
        )
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
                if let Some(child_layout) = layout.children().next() {
                    self.content.as_widget().draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        style,
                        child_layout,
                        cursor,
                        &clipped_viewport,
                    );
                }
            });
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        if let Some(child_layout) = layout.children().next() {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], child_layout, renderer, operation);
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

        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &clipped_viewport,
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
        let bounds = layout.bounds();
        let Some(clipped_viewport) = bounds.intersection(viewport) else {
            return mouse::Interaction::None;
        };

        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                child_layout,
                cursor,
                &clipped_viewport,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let child_layout = layout.children().next()?;
        let p = self.progress.clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - p).powi(3);
        let offset_y = -layout.bounds().height * (1.0 - eased);
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            child_layout,
            renderer,
            viewport,
            translation + Vector::new(0.0, offset_y),
        )
    }
}

impl<'a, Message, Theme, Renderer> From<SlideDown<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: SlideDown<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}
