// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Small animation wrappers for widgets.

use std::sync::RwLock;

use crate::Element;
use iced_anim::{Animated, Motion};
use iced_core::event::Event;
use iced_core::renderer::Renderer as _;
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Clipboard, Layout, Length, Rectangle, Shell, Size, Transformation, Vector, Widget, layout,
    mouse, overlay, renderer, window,
};

const HIDDEN: f32 = 0.0;
const VISIBLE: f32 = 1.0;
static MOTION: RwLock<Motion> = RwLock::new(Motion::SMOOTH);

pub(crate) fn set_motion(motion: Motion) {
    *MOTION.write().unwrap() = motion;
}

pub(crate) fn motion() -> Motion {
    *MOTION.read().unwrap()
}

/// The direction an animated widget moves as it appears.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
    /// Move upward from below.
    #[default]
    Up,
    /// Move downward from above.
    Down,
    /// Move left from the right.
    Left,
    /// Move right from the left.
    Right,
}

/// Wraps content in the default upward pop animation.
pub fn pop<'a, Message>(content: impl Into<Element<'a, Message>>) -> Pop<'a, Message> {
    Pop::new(content)
}

/// Wraps content in an upward pop animation.
pub fn pop_up<'a, Message>(content: impl Into<Element<'a, Message>>) -> Pop<'a, Message> {
    Pop::new(content).direction(Direction::Up)
}

/// Wraps content in a downward pop animation.
pub fn pop_down<'a, Message>(content: impl Into<Element<'a, Message>>) -> Pop<'a, Message> {
    Pop::new(content).direction(Direction::Down)
}

/// Wraps content in a leftward pop animation.
pub fn pop_left<'a, Message>(content: impl Into<Element<'a, Message>>) -> Pop<'a, Message> {
    Pop::new(content).direction(Direction::Left)
}

/// Wraps content in a rightward pop animation.
pub fn pop_right<'a, Message>(content: impl Into<Element<'a, Message>>) -> Pop<'a, Message> {
    Pop::new(content).direction(Direction::Right)
}

#[derive(Debug)]
struct State {
    animation: Animated<f32>,
    started: bool,
}

/// A widget that scales and translates its content as it appears.
pub struct Pop<'a, Message> {
    content: Element<'a, Message>,
    animation: Animated<f32>,
    direction: Direction,
}

impl<'a, Message> Pop<'a, Message> {
    /// Creates an upward pop animation around `content`.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            animation: Animated::spring(HIDDEN, motion()),
            direction: Direction::default(),
        }
    }

    /// Sets the animation used for the visibility progress.
    ///
    /// The wrapper manages the animation target: `0.0` is hidden and `1.0` is
    /// visible.
    #[must_use]
    pub fn animation(mut self, animation: Animated<f32>) -> Self {
        self.animation = animation;
        self
    }

    /// Sets the direction the content moves as it appears.
    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}

impl<Message: 'static> Widget<Message, crate::Theme, crate::Renderer> for Pop<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let mut animation = self.animation.clone();
        animation.settle_at(HIDDEN);

        tree::State::new(State {
            animation,
            started: false,
        })
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let transformation = transformation(
            layout.bounds(),
            *tree.state.downcast_ref::<State>().animation.value(),
            self.direction,
        );
        let inverse = transformation.inverse();
        renderer.with_transformation(transformation, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor * inverse,
                &(*viewport * inverse),
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let transformation = transformation(
            layout.bounds(),
            *tree.state.downcast_ref::<State>().animation.value(),
            self.direction,
        );
        let inverse = transformation.inverse();
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor * inverse,
            renderer,
            clipboard,
            shell,
            &(*viewport * inverse),
        );

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<State>();
            if !state.started {
                state.started = true;
                state.animation.set_target(VISIBLE);
                shell.request_redraw();
            } else if state.animation.is_animating() {
                shell.request_redraw();
                state.animation.tick(*now);
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        let transformation = transformation(
            layout.bounds(),
            *tree.state.downcast_ref::<State>().animation.value(),
            self.direction,
        );
        let inverse = transformation.inverse();
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor * inverse,
            &(*viewport * inverse),
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &crate::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }

    fn drag_destinations(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.content.as_widget().drag_destinations(
            &tree.children[0],
            layout,
            renderer,
            dnd_rectangles,
        );
    }
}

impl<'a, Message: 'static> From<Pop<'a, Message>> for Element<'a, Message> {
    fn from(pop: Pop<'a, Message>) -> Self {
        Element::new(pop)
    }
}

fn transformation(
    bounds: Rectangle,
    progress: f32,
    direction: Direction,
) -> Transformation {
    let progress = progress.clamp(HIDDEN, VISIBLE);
    let scale = 0.96 + 0.04 * progress;
    let offset = 24.0 * (1.0 - progress);
    let center = bounds.center();
    let offset = match direction {
        Direction::Up => Vector::new(0.0, offset),
        Direction::Down => Vector::new(0.0, -offset),
        Direction::Left => Vector::new(offset, 0.0),
        Direction::Right => Vector::new(-offset, 0.0),
    };

    Transformation::translate(center.x + offset.x, center.y + offset.y)
        * Transformation::scale(scale)
        * Transformation::translate(-center.x, -center.y)
}
