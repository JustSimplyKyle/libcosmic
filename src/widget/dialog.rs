use crate::iced::{Length, Pixels};
use crate::{Element, style, theme, widget};
use iced_anim::{Animated, Motion};
use iced_core::event::Event;
use iced_core::renderer::Renderer as _;
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Clipboard, Layout, Rectangle, Shell, Size, Transformation, Vector, Widget, layout, mouse,
    overlay, renderer, window,
};
use std::borrow::Cow;

pub fn dialog<'a, Message>() -> Dialog<'a, Message> {
    Dialog::new()
}

pub struct Dialog<'a, Message> {
    title: Option<Cow<'a, str>>,
    icon: Option<Element<'a, Message>>,
    body: Option<Cow<'a, str>>,
    controls: Vec<Element<'a, Message>>,
    primary_action: Option<Element<'a, Message>>,
    secondary_action: Option<Element<'a, Message>>,
    tertiary_action: Option<Element<'a, Message>>,
    width: Option<Length>,
    height: Option<Length>,
    max_width: Option<Pixels>,
    max_height: Option<Pixels>,
    is_overlay: bool,
    animation: Animated<f32>,
}

impl<Message> Default for Dialog<'_, Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> Dialog<'a, Message> {
    pub fn new() -> Self {
        Self {
            title: None,
            icon: None,
            body: None,
            controls: Vec::new(),
            primary_action: None,
            secondary_action: None,
            tertiary_action: None,
            width: None,
            height: None,
            max_width: None,
            max_height: None,
            is_overlay: true,
            animation: Animated::spring(0.0, Motion::SMOOTH),
        }
    }

    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn body(mut self, body: impl Into<Cow<'a, str>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn control(mut self, control: impl Into<Element<'a, Message>>) -> Self {
        self.controls.push(control.into());
        self
    }

    pub fn primary_action(mut self, button: impl Into<Element<'a, Message>>) -> Self {
        self.primary_action = Some(button.into());
        self
    }

    pub fn secondary_action(mut self, button: impl Into<Element<'a, Message>>) -> Self {
        self.secondary_action = Some(button.into());
        self
    }

    pub fn tertiary_action(mut self, button: impl Into<Element<'a, Message>>) -> Self {
        self.tertiary_action = Some(button.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = Some(max_height.into());
        self
    }

    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = Some(max_width.into());
        self
    }

    pub fn is_overlay(mut self, is_overlay: bool) -> Self {
        self.is_overlay = is_overlay;
        self
    }

    /// Sets the animation used when the dialog is shown.
    ///
    /// The animation value is treated as pop progress, where `0.0` is slightly
    /// smaller and lower than its final position and `1.0` is fully shown.
    #[must_use]
    pub fn animation(mut self, animation: Animated<f32>) -> Self {
        self.animation = animation;
        self
    }
}

impl<'a, Message: Clone + 'static> From<Dialog<'a, Message>> for Element<'a, Message> {
    fn from(dialog: Dialog<'a, Message>) -> Self {
        let cosmic_theme::Spacing {
            space_l,
            space_m,
            space_s,
            space_xxs,
            ..
        } = theme::active().cosmic().spacing;

        let mut content_col = widget::column::with_capacity(3 + dialog.controls.len() * 2);

        let mut should_space = if let Some(title) = dialog.title {
            content_col = content_col.push(widget::text::title3(title));
            true
        } else {
            false
        };

        if let Some(body) = dialog.body {
            if should_space {
                content_col = content_col
                    .push(widget::space::vertical().height(Length::Fixed(space_xxs.into())));
            }
            content_col = content_col.push(
                widget::container(widget::scrollable(widget::text::body(body))).max_height(300.),
            );
            should_space = true;
        }
        for control in dialog.controls {
            if should_space {
                content_col = content_col
                    .push(widget::space::vertical().height(Length::Fixed(space_s.into())));
            }
            content_col = content_col.push(control);
            should_space = true;
        }

        let mut content_row = widget::row::with_capacity(2).spacing(space_s);
        if let Some(icon) = dialog.icon {
            content_row = content_row.push(icon);
        }
        content_row = content_row.push(content_col);

        let mut button_row = widget::row::with_capacity(4).spacing(space_xxs);
        if let Some(button) = dialog.tertiary_action {
            button_row = button_row.push(button);
        }
        button_row = button_row.push(widget::space::horizontal());
        if let Some(button) = dialog.secondary_action {
            button_row = button_row.push(button);
        }
        if let Some(button) = dialog.primary_action {
            button_row = button_row.push(button);
        }

        let mut container = widget::container(
            widget::column::with_children([content_row.into(), button_row.into()]).spacing(space_l),
        )
        .class(style::Container::Dialog(dialog.is_overlay))
        .padding(space_l)
        .width(dialog.width.unwrap_or(Length::Fixed(570.0)));

        if let Some(height) = dialog.height {
            container = container.height(height);
        }

        if let Some(max_width) = dialog.max_width {
            container = container.max_width(max_width);
        }

        if let Some(max_height) = dialog.max_height {
            container = container.max_height(max_height);
        }

        Pop::new(container, dialog.animation).into()
    }
}

#[derive(Debug)]
struct PopState {
    animation: Animated<f32>,
}

/// An internal wrapper that provides a dialog with persistent pop-in state.
struct Pop<'a, Message> {
    content: Element<'a, Message>,
    animation: Animated<f32>,
}

impl<'a, Message> Pop<'a, Message> {
    fn new(content: impl Into<Element<'a, Message>>, animation: Animated<f32>) -> Self {
        Self {
            content: content.into(),
            animation,
        }
    }
}

impl<Message: 'static> Widget<Message, crate::Theme, crate::Renderer> for Pop<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PopState>()
    }

    fn state(&self) -> tree::State {
        let mut animation = self.animation.clone();
        animation.settle_at(0.0);
        animation.set_target(1.0);
        tree::State::new(PopState { animation })
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
        let transformation = pop_transformation(
            layout.bounds(),
            *tree.state.downcast_ref::<PopState>().animation.value(),
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
        let transformation = pop_transformation(
            layout.bounds(),
            *tree.state.downcast_ref::<PopState>().animation.value(),
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
            let state = tree.state.downcast_mut::<PopState>();
            if state.animation.is_animating() {
                state.animation.tick(*now);
                shell.request_redraw();
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
        let transformation = pop_transformation(
            layout.bounds(),
            *tree.state.downcast_ref::<PopState>().animation.value(),
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

fn pop_transformation(bounds: Rectangle, progress: f32) -> Transformation {
    let progress = progress.clamp(0.0, 1.0);
    let scale = 0.96 + 0.04 * progress;
    let offset_y = 24.0 * (1.0 - progress);
    let center = bounds.center();

    Transformation::translate(center.x, center.y + offset_y)
        * Transformation::scale(scale)
        * Transformation::translate(-center.x, -center.y)
}
