/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Interactive Cursor Widget
 *
 * Wraps arbitrary Iced elements to report custom mouse cursor shapes, such as pointer
 * cursors for clickable seekbars and playback controls.
 */

use iced::advanced::widget::{Tree, Widget};
use iced::advanced::{Clipboard, Layout, Shell, layout, renderer};
use iced::mouse;
use iced::{Element, Event, Length, Point, Rectangle, Size};

pub struct PointerCursor<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> PointerCursor<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PointerCursor<'a, Message, Theme, Renderer>
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
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let child_interaction = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );
        match child_interaction {
            mouse::Interaction::Grab | mouse::Interaction::Grabbing => mouse::Interaction::Pointer,
            mouse::Interaction::None if cursor.is_over(layout.bounds()) => {
                mouse::Interaction::Pointer
            }
            other => other,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<PointerCursor<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: PointerCursor<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}

pub fn determine_resize_direction(
    bounds: Rectangle,
    pos: Point,
    border_margin: f32,
    corner_margin: f32,
) -> Option<iced::window::Direction> {
    if !bounds.contains(pos) {
        return None;
    }

    let on_top = pos.y < bounds.y + border_margin;
    let on_bottom = pos.y >= bounds.y + bounds.height - border_margin;
    let on_left = pos.x < bounds.x + border_margin;
    let on_right = pos.x >= bounds.x + bounds.width - border_margin;

    let near_top = pos.y < bounds.y + corner_margin;
    let near_bottom = pos.y >= bounds.y + bounds.height - corner_margin;
    let near_left = pos.x < bounds.x + corner_margin;
    let near_right = pos.x >= bounds.x + bounds.width - corner_margin;

    if (on_top && near_left) || (near_top && on_left) {
        Some(iced::window::Direction::NorthWest)
    } else if (on_top && near_right) || (near_top && on_right) {
        Some(iced::window::Direction::NorthEast)
    } else if (on_bottom && near_left) || (near_bottom && on_left) {
        Some(iced::window::Direction::SouthWest)
    } else if (on_bottom && near_right) || (near_bottom && on_right) {
        Some(iced::window::Direction::SouthEast)
    } else if on_top {
        Some(iced::window::Direction::North)
    } else if on_bottom {
        Some(iced::window::Direction::South)
    } else if on_left {
        Some(iced::window::Direction::West)
    } else if on_right {
        Some(iced::window::Direction::East)
    } else {
        None
    }
}

pub fn direction_to_interaction(direction: iced::window::Direction) -> mouse::Interaction {
    match direction {
        iced::window::Direction::North | iced::window::Direction::South => {
            mouse::Interaction::ResizingVertically
        }
        iced::window::Direction::East | iced::window::Direction::West => {
            mouse::Interaction::ResizingHorizontally
        }
        iced::window::Direction::NorthWest | iced::window::Direction::SouthEast => {
            mouse::Interaction::ResizingDiagonallyDown
        }
        iced::window::Direction::NorthEast | iced::window::Direction::SouthWest => {
            mouse::Interaction::ResizingDiagonallyUp
        }
    }
}

pub struct WindowBorderResizer<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_resize: Box<dyn Fn(iced::window::Direction) -> Message + 'a>,
    border_margin: f32,
    corner_margin: f32,
}

impl<'a, Message: 'a, Theme: 'a, Renderer: 'a> WindowBorderResizer<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        on_resize: impl Fn(iced::window::Direction) -> Message + 'a,
    ) -> Self {
        Self {
            content: content.into(),
            on_resize: Box::new(on_resize),
            border_margin: 6.0,
            corner_margin: 14.0,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for WindowBorderResizer<'a, Message, Theme, Renderer>
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
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(pos) = cursor.position_over(layout.bounds()) {
                if let Some(direction) = determine_resize_direction(
                    layout.bounds(),
                    pos,
                    self.border_margin,
                    self.corner_margin,
                ) {
                    shell.publish((self.on_resize)(direction));
                    shell.capture_event();
                    return;
                }
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position_over(layout.bounds()) {
            if let Some(direction) = determine_resize_direction(
                layout.bounds(),
                pos,
                self.border_margin,
                self.corner_margin,
            ) {
                return direction_to_interaction(direction);
            }
        }

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<WindowBorderResizer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: WindowBorderResizer<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}

