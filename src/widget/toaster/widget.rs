// Copyright 2024 wiiznokes
// SPDX-License-Identifier: MPL-2.0

use iced::{Limits, Size};
use iced_core::layout::Node;

use iced_anim::{Animated, Motion};
use iced_core::event::Event;
use iced_core::renderer::{self};
use iced_core::widget::Operation;
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Overlay, Point, Rectangle, Shell, Vector, Widget, layout,
    mouse, overlay, window,
};

const HIDDEN: f32 = 0.0;
const VISIBLE: f32 = 1.0;

#[derive(Debug)]
struct State {
    animation: Animated<f32>,
    is_empty: bool,
}

pub struct Toaster<'a, Message, Theme, Renderer> {
    toasts: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    is_empty: bool,
    animation: Animated<f32>,
}

impl<'a, Message, Theme, Renderer> Toaster<'a, Message, Theme, Renderer> {
    pub fn new(
        toasts: Element<'a, Message, Theme, Renderer>,
        content: Element<'a, Message, Theme, Renderer>,
        is_empty: bool,
    ) -> Self {
        Self {
            toasts,
            content,
            is_empty,
            animation: Animated::spring(HIDDEN, Motion::SMOOTH),
        }
    }

    /// Sets the animation used when the toaster is shown.
    ///
    /// The animation value is treated as a visibility progress, where `0.0` is
    /// hidden and `1.0` is fully visible. Its target is managed by the toaster.
    #[must_use]
    pub fn animation(mut self, animation: Animated<f32>) -> Self {
        self.animation = animation;
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toaster<'_, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let mut animation = self.animation.clone();
        animation.set_target(if self.is_empty { HIDDEN } else { VISIBLE });

        tree::State::new(State {
            animation,
            is_empty: self.is_empty,
        })
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
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

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.toasts)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        if state.is_empty != self.is_empty {
            state.is_empty = self.is_empty;
            state
                .animation
                .set_target(if self.is_empty { HIDDEN } else { VISIBLE });
        }

        tree.diff_children(&mut [&mut self.content, &mut self.toasts]);
    }

    fn operate<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut state.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = state.state.downcast_mut::<State>();
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if state.animation.is_animating() {
                state.animation.tick(*now);
                shell.invalidate_layout();
                shell.request_redraw();
            }
        }
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let animation = &state.state.downcast_ref::<State>().animation;

        //TODO: this hides the overlay of the content during the toast
        if self.is_empty && !animation.is_animating() {
            self.content.as_widget_mut().overlay(
                &mut state.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        } else {
            let progress = *animation.value();

            Some(overlay::Element::new(Box::new(ToasterOverlay::new(
                &mut state.children[1],
                &mut self.toasts,
                progress,
            ))))
        }
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.content.as_widget().drag_destinations(
            &state.children[0],
            layout,
            renderer,
            dnd_rectangles,
        );
    }
}

struct ToasterOverlay<'a, 'b, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    state: &'b mut Tree,
    element: &'b mut Element<'a, Message, Theme, Renderer>,
    progress: f32,
}

impl<'a, 'b, Message, Theme, Renderer> ToasterOverlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn new(
        state: &'b mut Tree,
        element: &'b mut Element<'a, Message, Theme, Renderer>,
        progress: f32,
    ) -> Self {
        Self {
            state,
            element,
            progress,
        }
    }
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for ToasterOverlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let limits = Limits::new(Size::ZERO, bounds);

        let node = self
            .element
            .as_widget_mut()
            .layout(self.state, renderer, &limits);

        let offset = 15.;

        let position = Point::new(
            (bounds.width / 2.) - (node.size().width / 2.),
            bounds.height - (node.size().height + offset) * self.progress,
        );

        node.move_to(position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        self.element
            .as_widget()
            .draw(self.state, renderer, theme, style, layout, cursor, &bounds);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) {
        self.element.as_widget_mut().update(
            self.state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element.as_widget().mouse_interaction(
            self.state,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.element.as_widget_mut().overlay(
            self.state,
            layout,
            renderer,
            &layout.bounds(),
            Default::default(),
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Toaster<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Theme: 'a,
    Message: 'a,
{
    fn from(toaster: Toaster<'a, Message, Theme, Renderer>) -> Self {
        Element::new(toaster)
    }
}
