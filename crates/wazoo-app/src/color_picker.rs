/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Interactive HSV Color Picker Widgets
 *
 * Custom Iced widgets for continuous 2D Saturation-Value selection (`ColorSvPad`),
 * 1D Rainbow Hue selection (`HueBar`), and HSV <-> RGB conversion utilities.
 */

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::gradient::{Gradient, Linear};
use iced::mouse;
use iced::{Background, Border, Color, Element, Event, Length, Radians, Rectangle, Shadow, Size};
use std::f32::consts::{FRAC_PI_2, PI};

/// Converts an `(r, g, b)` byte triple into `(hue_deg [0..360), saturation [0..1], value [0..1])`.
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let v = max;
    let s = if max > 0.0 { delta / max } else { 0.0 };

    let h = if delta <= 1e-6 {
        0.0
    } else if (max - rf).abs() < 1e-6 {
        60.0 * (((gf - bf) / delta).rem_euclid(6.0))
    } else if (max - gf).abs() < 1e-6 {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    (h.rem_euclid(360.0), s.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

/// Converts `(hue_deg [0..360], saturation [0..1], value [0..1])` into `(r, g, b)` bytes.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// Returns the fully saturated `iced::Color` for a given hue in degrees `[0..360)`.
pub fn hue_to_color(h: f32) -> Color {
    let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);
    Color::from_rgb8(r, g, b)
}

#[derive(Debug, Default)]
struct DragState {
    is_dragging: bool,
}

/// 2D Saturation (X) × Value (Y) gradient color pad.
pub struct ColorSvPad<'a, Message> {
    hue: f32,
    saturation: f32,
    value: f32,
    height: f32,
    border_color: Color,
    radius: f32,
    on_change: Box<dyn Fn(f32, f32) -> Message + 'a>,
    on_commit: Option<Message>,
}

impl<'a, Message> ColorSvPad<'a, Message> {
    pub fn new(
        hue: f32,
        saturation: f32,
        value: f32,
        on_change: impl Fn(f32, f32) -> Message + 'a,
    ) -> Self {
        Self {
            hue,
            saturation: saturation.clamp(0.0, 1.0),
            value: value.clamp(0.0, 1.0),
            height: 130.0,
            border_color: Color::from_rgb(0.28, 0.28, 0.28),
            radius: 6.0,
            on_change: Box::new(on_change),
            on_commit: None,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn on_commit(mut self, msg: Message) -> Self {
        self.on_commit = Some(msg);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ColorSvPad<'a, Message>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DragState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DragState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fixed(self.height))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        let pure_hue = hue_to_color(self.hue);

        // Layer 1: Horizontal gradient White (S=0) -> Pure Hue (S=1)
        let sat_gradient = Gradient::Linear(
            Linear::new(Radians(FRAC_PI_2))
                .add_stop(0.0, Color::WHITE)
                .add_stop(1.0, pure_hue),
        );
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    radius: self.radius.into(),
                    ..Default::default()
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Gradient(sat_gradient),
        );

        // Layer 2: Vertical gradient Transparent (V=1) -> Black (V=0)
        let val_gradient = Gradient::Linear(
            Linear::new(Radians(PI))
                .add_stop(0.0, Color::TRANSPARENT)
                .add_stop(1.0, Color::BLACK),
        );
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    radius: self.radius.into(),
                    width: 1.0,
                    color: self.border_color,
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Gradient(val_gradient),
        );

        // Reticle indicator at (saturation, 1.0 - value)
        let cx = bounds.x + self.saturation * bounds.width;
        let cy = bounds.y + (1.0 - self.value) * bounds.height;

        // Outer dark ring
        renderer.fill_quad(
            Quad {
                bounds: Rectangle {
                    x: cx - 7.0,
                    y: cy - 7.0,
                    width: 14.0,
                    height: 14.0,
                },
                border: Border {
                    radius: 7.0.into(),
                    width: 2.0,
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.85),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            Background::Color(Color::TRANSPARENT),
        );

        // Inner white ring + selected color fill
        let (r, g, b) = hsv_to_rgb(self.hue, self.saturation, self.value);
        renderer.fill_quad(
            Quad {
                bounds: Rectangle {
                    x: cx - 5.5,
                    y: cy - 5.5,
                    width: 11.0,
                    height: 11.0,
                },
                border: Border {
                    radius: 5.5.into(),
                    width: 2.0,
                    color: Color::WHITE,
                },
                shadow: Shadow::default(),
                snap: false,
            },
            Background::Color(Color::from_rgb8(r, g, b)),
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<DragState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_over(bounds) {
                    state.is_dragging = true;
                    let s = ((pos.x - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0);
                    let v = (1.0 - (pos.y - bounds.y) / bounds.height.max(1.0)).clamp(0.0, 1.0);
                    shell.publish((self.on_change)(s, v));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                if let Some(pos) = cursor.position() {
                    let s = ((pos.x - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0);
                    let v = (1.0 - (pos.y - bounds.y) / bounds.height.max(1.0)).clamp(0.0, 1.0);
                    shell.publish((self.on_change)(s, v));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.is_dragging =>
            {
                state.is_dragging = false;
                if let Some(ref msg) = self.on_commit {
                    shell.publish(msg.clone());
                }
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<DragState>();
        if state.is_dragging || cursor.is_over(layout.bounds()) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ColorSvPad<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(pad: ColorSvPad<'a, Message>) -> Self {
        Element::new(pad)
    }
}

/// 1D Rainbow Hue spectrum bar `[0°..360°]`.
pub struct HueBar<Message> {
    hue: f32,
    height: f32,
    border_color: Color,
    radius: f32,
    on_change: Box<dyn Fn(f32) -> Message>,
    on_commit: Option<Message>,
}

impl<Message> HueBar<Message> {
    pub fn new(hue: f32, on_change: impl Fn(f32) -> Message + 'static) -> Self {
        Self {
            hue: hue.rem_euclid(360.0),
            height: 18.0,
            border_color: Color::from_rgb(0.28, 0.28, 0.28),
            radius: 6.0,
            on_change: Box::new(on_change),
            on_commit: None,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn on_commit(mut self, msg: Message) -> Self {
        self.on_commit = Some(msg);
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for HueBar<Message>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DragState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DragState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fixed(self.height))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        let hue_gradient = Gradient::Linear(
            Linear::new(Radians(FRAC_PI_2))
                .add_stop(0.0, Color::from_rgb(1.0, 0.0, 0.0))
                .add_stop(1.0 / 6.0, Color::from_rgb(1.0, 1.0, 0.0))
                .add_stop(2.0 / 6.0, Color::from_rgb(0.0, 1.0, 0.0))
                .add_stop(3.0 / 6.0, Color::from_rgb(0.0, 1.0, 1.0))
                .add_stop(4.0 / 6.0, Color::from_rgb(0.0, 0.0, 1.0))
                .add_stop(5.0 / 6.0, Color::from_rgb(1.0, 0.0, 1.0))
                .add_stop(1.0, Color::from_rgb(1.0, 0.0, 0.0)),
        );

        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    radius: self.radius.into(),
                    width: 1.0,
                    color: self.border_color,
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Gradient(hue_gradient),
        );

        let ratio = (self.hue.rem_euclid(360.0) / 360.0).clamp(0.0, 1.0);
        let hx = bounds.x + ratio * bounds.width;
        let thumb_w = 8.0;
        let thumb_h = bounds.height + 4.0;

        renderer.fill_quad(
            Quad {
                bounds: Rectangle {
                    x: (hx - thumb_w / 2.0).clamp(bounds.x, bounds.x + bounds.width - thumb_w),
                    y: bounds.y - 2.0,
                    width: thumb_w,
                    height: thumb_h,
                },
                border: Border {
                    radius: 4.0.into(),
                    width: 2.0,
                    color: Color::WHITE,
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.65),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 3.0,
                },
                snap: false,
            },
            Background::Color(hue_to_color(self.hue)),
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<DragState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_over(bounds) {
                    state.is_dragging = true;
                    let ratio = ((pos.x - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0);
                    shell.publish((self.on_change)(ratio * 359.99));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                if let Some(pos) = cursor.position() {
                    let ratio = ((pos.x - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0);
                    shell.publish((self.on_change)(ratio * 359.99));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.is_dragging =>
            {
                state.is_dragging = false;
                if let Some(ref msg) = self.on_commit {
                    shell.publish(msg.clone());
                }
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<DragState>();
        if state.is_dragging || cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message, Theme, Renderer> From<HueBar<Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(bar: HueBar<Message>) -> Self {
        Element::new(bar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_hsv_roundtrip() {
        let samples = [
            (0x42, 0xb8, 0x83),
            (0x1f, 0x09, 0x2f),
            (0x1f, 0x1f, 0x1f),
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (255, 255, 255),
            (0, 0, 0),
        ];
        for (r, g, b) in samples {
            let (h, s, v) = rgb_to_hsv(r, g, b);
            let (rr, gg, bb) = hsv_to_rgb(h, s, v);
            assert!(
                (r as i16 - rr as i16).abs() <= 1
                    && (g as i16 - gg as i16).abs() <= 1
                    && (b as i16 - bb as i16).abs() <= 1,
                "RGB->HSV->RGB mismatch for ({r},{g},{b}) -> ({h},{s},{v}) -> ({rr},{gg},{bb})"
            );
        }
    }
}
