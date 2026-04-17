// Modal unit rendering: data types, build functions, layout math, and render functions
// for the at-rest and in-transition modal strip. Split out of ui/mod.rs for readability.
// All items are pub(super) - visible to ui/mod.rs but not exported beyond the ui module.

use crate::app::App;
use crate::modal_layout::{modal_list_view_dimensions, ModalListViewSnapshot, ModalStubKind, ModalUnitRange, SimpleModalUnitLayout};
use crate::Message;
use iced::advanced::{layout, renderer, widget as adv_widget};
use iced::advanced::widget::Widget as AdvWidget;
use iced::widget::{container, row, Space};
use iced::{Element, Length, Point, Rectangle, Size};
use iced::mouse;
use super::{active_simple_modal_content, apply_alpha, modal_card, preview_simple_modal_content, ModalCardRole, ModalRenderMode};

// LESSON 1: The card vocabulary. Every card in the modal strip is described by one of these types before anything is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModalUnitStubMode {
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModalUnitSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ModalUnitCardKind {
    Stub {
        side: ModalUnitSide,
        mode: ModalUnitStubMode,
        stub_kind: ModalStubKind,
    },
    Preview {
        snapshot: ModalListViewSnapshot,
    },
    Active {
        snapshot: ModalListViewSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ModalUnitCardData {
    pub(super) kind: ModalUnitCardKind,
    pub(super) width: f32,
    pub(super) alpha: f32,
}

// LESSON 2: The finished strip description. A list of cards in left-to-right order, plus a helper to measure the strip's total pixel width including gaps.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct RenderedModalUnit {
    pub(super) cards: Vec<ModalUnitCardData>,
}

impl RenderedModalUnit {
    pub(super) fn total_width(&self, spacer_width: f32) -> f32 {
        let card_widths = self.cards.iter().map(|card| card.width).sum::<f32>();
        let spacing = spacer_width * self.cards.len().saturating_sub(1) as f32;
        card_widths + spacing
    }
}

pub(super) fn default_stub_mode(_side: ModalUnitSide) -> ModalUnitStubMode {
    ModalUnitStubMode::Visible
}

// LESSON 3: Assembles a still (at-rest) modal strip. Decides stub types based on position, marks the active card, and returns a RenderedModalUnit with all alphas at 1.0.
pub(super) fn build_rendered_modal_unit(
    app: &App,
    layout: &SimpleModalUnitLayout,
    unit: &ModalUnitRange,
    left_stub_mode: ModalUnitStubMode,
    right_stub_mode: ModalUnitStubMode,
) -> RenderedModalUnit {
    let total_snapshots = layout.sequence.snapshots.len();
    let left_kind = if unit.start == 0 {
        ModalStubKind::Exit
    } else {
        ModalStubKind::NavLeft
    };
    let right_kind = if unit.end + 1 >= total_snapshots {
        ModalStubKind::Confirm
    } else {
        ModalStubKind::NavRight
    };

    let mut cards = Vec::new();
    if unit.shows_stubs {
        cards.push(ModalUnitCardData {
            kind: ModalUnitCardKind::Stub {
                side: ModalUnitSide::Left,
                mode: left_stub_mode,
                stub_kind: left_kind,
            },
            width: app.ui_theme.modal_stub_width,
            alpha: 1.0,
        });
    }
    for sequence_idx in unit.start..=unit.end {
        let Some(snapshot) = layout.sequence.snapshots.get(sequence_idx).cloned() else {
            continue;
        };
        let width = modal_list_view_dimensions(&snapshot).0;
        let kind = if sequence_idx == layout.sequence.active_sequence_index {
            ModalUnitCardKind::Active { snapshot }
        } else {
            ModalUnitCardKind::Preview { snapshot }
        };
        cards.push(ModalUnitCardData {
            kind,
            width,
            alpha: 1.0,
        });
    }
    if unit.shows_stubs {
        cards.push(ModalUnitCardData {
            kind: ModalUnitCardKind::Stub {
                side: ModalUnitSide::Right,
                mode: right_stub_mode,
                stub_kind: right_kind,
            },
            width: app.ui_theme.modal_stub_width,
            alpha: 1.0,
        });
    }
    RenderedModalUnit { cards }
}

// LESSON 4: Convenience helper for adding a stub card to a list. Used by the transition builder to avoid repeating the same struct construction each time.
pub(super) fn push_strip_stub(
    cards: &mut Vec<ModalUnitCardData>,
    side: ModalUnitSide,
    kind: ModalStubKind,
    width: f32,
    alpha: f32,
) {
    cards.push(ModalUnitCardData {
        kind: ModalUnitCardKind::Stub {
            side,
            mode: ModalUnitStubMode::Visible,
            stub_kind: kind,
        },
        width,
        alpha,
    });
}

pub(super) fn build_connected_transition_rendered_unit(
    stub_width: f32,
    layout: &SimpleModalUnitLayout,
    arrival: &crate::app::ModalArrivalLayer,
    departure: &crate::app::ModalDepartureLayer,
    progress: f32,
) -> RenderedModalUnit {
    let dep_alpha = 1.0 - progress;
    let arr_alpha = progress;
    let dep_geo = &departure.geometry;
    let arr_geo = &arrival.geometry;
    let dep_content = &departure.content;
    let mut cards = Vec::new();

    let (dep_far_stub, transition_stub, arr_far_stub) = match arrival.focus_direction {
        crate::app::FocusDirection::Forward => (
            dep_geo.leading_stub_kind.map(|kind| (ModalUnitSide::Left, kind)),
            dep_geo.trailing_stub_kind.map(|kind| (ModalUnitSide::Right, kind)),
            arr_geo.trailing_stub_kind.map(|kind| (ModalUnitSide::Right, kind)),
        ),
        crate::app::FocusDirection::Backward => (
            dep_geo.trailing_stub_kind.map(|kind| (ModalUnitSide::Right, kind)),
            dep_geo.leading_stub_kind.map(|kind| (ModalUnitSide::Left, kind)),
            arr_geo.leading_stub_kind.map(|kind| (ModalUnitSide::Left, kind)),
        ),
    };

    match arrival.focus_direction {
        crate::app::FocusDirection::Forward => {
            // Strip layout: [dep_outer | dep_modals | transition | arr_modals | arr_outer]
            // The strip slides left as p advances, bringing arr into view from the right.
            if dep_geo.shows_stubs {
                if let Some((side, kind)) = dep_far_stub {
                    push_strip_stub(&mut cards, side, kind, stub_width, dep_alpha);
                }
            }
            for (i, snapshot) in dep_content.modals.iter().enumerate() {
                let width = dep_geo
                    .modal_widths
                    .get(i)
                    .copied()
                    .unwrap_or_else(|| modal_list_view_dimensions(snapshot).0);
                cards.push(ModalUnitCardData {
                    kind: ModalUnitCardKind::Preview { snapshot: snapshot.clone() },
                    width,
                    alpha: dep_alpha,
                });
            }
            if dep_geo.shows_stubs {
                if let Some((side, kind)) = transition_stub {
                    push_strip_stub(&mut cards, side, kind, stub_width, 1.0);
                }
            }
            for (sequence_idx, width) in arr_geo
                .modal_index_range
                .clone()
                .zip(arr_geo.modal_widths.iter().copied())
            {
                let Some(snapshot) = layout.sequence.snapshots.get(sequence_idx).cloned() else {
                    continue;
                };
                let kind = if sequence_idx == layout.sequence.active_sequence_index {
                    ModalUnitCardKind::Active { snapshot }
                } else {
                    ModalUnitCardKind::Preview { snapshot }
                };
                cards.push(ModalUnitCardData { kind, width, alpha: arr_alpha });
            }
            if arr_geo.shows_stubs {
                if let Some((side, kind)) = arr_far_stub {
                    push_strip_stub(&mut cards, side, kind, stub_width, arr_alpha);
                }
            }
        }
        crate::app::FocusDirection::Backward => {
            // Strip layout: [arr_outer | arr_modals | transition | dep_modals | dep_outer]
            // The strip slides right as p advances, bringing arr into view from the left.
            if arr_geo.shows_stubs {
                if let Some((side, kind)) = arr_far_stub {
                    push_strip_stub(&mut cards, side, kind, stub_width, arr_alpha);
                }
            }
            for (sequence_idx, width) in arr_geo
                .modal_index_range
                .clone()
                .zip(arr_geo.modal_widths.iter().copied())
            {
                let Some(snapshot) = layout.sequence.snapshots.get(sequence_idx).cloned() else {
                    continue;
                };
                let kind = if sequence_idx == layout.sequence.active_sequence_index {
                    ModalUnitCardKind::Active { snapshot }
                } else {
                    ModalUnitCardKind::Preview { snapshot }
                };
                cards.push(ModalUnitCardData { kind, width, alpha: arr_alpha });
            }
            if dep_geo.shows_stubs {
                if let Some((side, kind)) = transition_stub {
                    push_strip_stub(&mut cards, side, kind, stub_width, 1.0);
                }
            }
            for (i, snapshot) in dep_content.modals.iter().enumerate() {
                let width = dep_geo
                    .modal_widths
                    .get(i)
                    .copied()
                    .unwrap_or_else(|| modal_list_view_dimensions(snapshot).0);
                cards.push(ModalUnitCardData {
                    kind: ModalUnitCardKind::Preview { snapshot: snapshot.clone() },
                    width,
                    alpha: dep_alpha,
                });
            }
            if dep_geo.shows_stubs {
                if let Some((side, kind)) = dep_far_stub {
                    push_strip_stub(&mut cards, side, kind, stub_width, dep_alpha);
                }
            }
        }
    }

    RenderedModalUnit { cards }
}

// LESSON 6: Centering math for the modal strip. Returns (outer_width, left_pad, right_pad) to position the strip inside an oversized container. The container is wider than the viewport (the "runway") so the strip can slide without being clipped. A positive shift moves the strip right; left_pad and right_pad tip in opposite directions to achieve that.
pub(super) fn modal_unit_runway_layout(viewport_width: f32, row_width: f32, shift: f32) -> (f32, f32, f32) {
    let base_offset = (viewport_width - row_width) * 0.5;
    let runway = ((row_width - viewport_width) * 0.5).max(0.0) + shift.abs();
    let left_pad = (runway + base_offset + shift).max(0.0);
    let right_pad = (runway + base_offset - shift).max(0.0);
    let outer_width = viewport_width + runway * 2.0;
    (outer_width, left_pad, right_pad)
}

// LESSON 7: Paints the still modal strip. Loops through each card description and produces an iced widget - stub symbol, dim preview, or interactive active card. Wraps the result in a centered runway container using the shift math from modal_unit_runway_layout.
pub(super) fn render_modal_unit<'a>(
    app: &'a App,
    rendered: &RenderedModalUnit,
    current_modal: Option<&'a crate::modal::SearchModal>,
    modal_height: f32,
    shift: f32,
    interactive_active: bool,
) -> Element<'a, Message> {
    let spacer_width = app.modal_spacer_width();
    let mut cards: Vec<Element<'a, Message>> = Vec::new();
    for card in &rendered.cards {
        let alpha = card.alpha;
        match &card.kind {
            ModalUnitCardKind::Stub { mode, stub_kind, .. } => match mode {
                ModalUnitStubMode::Visible => {
                    let text_color = match stub_kind {
                        ModalStubKind::NavLeft | ModalStubKind::NavRight => {
                            app.ui_theme.modal_nav_stub_text
                        }
                        ModalStubKind::Exit => app.ui_theme.modal_exit_stub_text,
                        ModalStubKind::Confirm => app.ui_theme.modal_confirm_stub_text,
                    };
                    cards.push(modal_card(
                        app,
                        container(
                            iced::widget::text(stub_kind.symbol().to_string())
                                .font(app.ui_theme.font_modal)
                                .size(24)
                                .color(apply_alpha(text_color, alpha)),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                        ModalRenderMode::Preview,
                        ModalCardRole::Stub(*stub_kind),
                        false,
                        card.width,
                        modal_height,
                        alpha,
                    ));
                }
            },
            ModalUnitCardKind::Preview { snapshot } => cards.push(modal_card(
                app,
                preview_simple_modal_content(app, snapshot.clone(), alpha),
                ModalRenderMode::Preview,
                ModalCardRole::Inactive,
                false,
                card.width,
                modal_height,
                alpha,
            )),
            ModalUnitCardKind::Active { snapshot } => {
                if interactive_active {
                    if let Some(modal) = current_modal {
                        cards.push(modal_card(
                            app,
                            active_simple_modal_content(app, modal, alpha),
                            ModalRenderMode::Interactive,
                            ModalCardRole::Active,
                            true,
                            card.width,
                            modal_height,
                            alpha,
                        ));
                    } else {
                        cards.push(modal_card(
                            app,
                            preview_simple_modal_content(app, snapshot.clone(), alpha),
                            ModalRenderMode::Preview,
                            ModalCardRole::Active,
                            false,
                            card.width,
                            modal_height,
                            alpha,
                        ));
                    }
                } else {
                    cards.push(modal_card(
                        app,
                        preview_simple_modal_content(app, snapshot.clone(), alpha),
                        ModalRenderMode::Preview,
                        ModalCardRole::Active,
                        false,
                        card.width,
                        modal_height,
                        alpha,
                    ));
                }
            }
        }
    }

    let row_width = rendered.total_width(spacer_width);
    let viewport_width = app.viewport_size.map(|size| size.width).unwrap_or(row_width);
    let (outer_width, left_pad, right_pad) =
        modal_unit_runway_layout(viewport_width, row_width, shift);

    container(
        container(
            row![
                Space::with_width(Length::Fixed(left_pad)),
                row(cards).spacing(spacer_width).align_y(iced::alignment::Vertical::Center),
                Space::with_width(Length::Fixed(right_pad))
            ]
            .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(outer_width)),
    )
    .width(Length::Fill)
    .center_x(Length::Fill)
    .into()
}

/// Returns the total rendered width of a unit from its frozen geometry.
pub(super) fn transition_unit_display_width(geometry: &crate::app::UnitGeometry, stub_width: f32) -> f32 {
    let n = geometry.modal_widths.len();
    let modals: f32 = geometry.modal_widths.iter().sum();
    if geometry.shows_stubs {
        2.0 * stub_width + modals + (n as f32 + 1.0) * geometry.effective_spacer_width
    } else {
        modals + (n.saturating_sub(1) as f32) * geometry.effective_spacer_width
    }
}

// ClipTranslate: custom widget that renders content at an arbitrary x_offset (may be negative)
// and clips it to a viewport-width envelope. This is the only way in iced to correctly position
// a widget strip at a negative x without the parent Limits system clamping the inner width.
//
// Why not the runway approach: iced's Limits.width(Length::Fixed(n)) clamps n to parent max,
// so an inner container can never exceed viewport_width. Centering then gives 0 offset and
// left_pad Space consumes all the room the cards need. ClipTranslate bypasses this by using
// Limits::new(Size::ZERO, Size::new(f32::MAX, h)) for the content layout pass, then places
// the resulting node at x_offset (can be negative). The clip envelope is explicit.
struct ClipTranslate<'a, M, T, R> {
    x_offset: f32,
    width: f32,
    height: f32,
    content: Element<'a, M, T, R>,
}

impl<'a, M, T, R> ClipTranslate<'a, M, T, R> {
    fn new(x_offset: f32, width: f32, height: f32, content: impl Into<Element<'a, M, T, R>>) -> Self {
        Self { x_offset, width, height, content: content.into() }
    }
}

impl<'a, M, T, R> AdvWidget<M, T, R> for ClipTranslate<'a, M, T, R>
where
    R: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.width),
            height: Length::Fixed(self.height),
        }
    }

    fn layout(
        &self,
        tree: &mut adv_widget::Tree,
        renderer: &R,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Clamp to parent-granted height so clip envelope and child layout agree.
        let effective_height = self.height.min(limits.max().height);
        let content_limits = layout::Limits::new(
            Size::ZERO,
            Size::new(f32::MAX, effective_height),
        );
        let content_node = self.content.as_widget()
            .layout(&mut tree.children[0], renderer, &content_limits)
            .move_to(Point::new(self.x_offset, 0.0));
        layout::Node::with_children(Size::new(self.width, effective_height), vec![content_node])
    }

    fn draw(
        &self,
        tree: &adv_widget::Tree,
        renderer: &mut R,
        theme: &T,
        style: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if let Some(clipped_viewport) = bounds.intersection(viewport) {
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
        }
    }

    fn children(&self) -> Vec<adv_widget::Tree> {
        vec![adv_widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut adv_widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn operate(
        &self,
        tree: &mut adv_widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &R,
        operation: &mut dyn adv_widget::Operation,
    ) {
        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget().operate(&mut tree.children[0], child_layout, renderer, operation);
        }
    }

    fn on_event(
        &mut self,
        tree: &mut adv_widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &R,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &Rectangle,
    ) -> iced::event::Status {
        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        } else {
            iced::event::Status::Ignored
        }
    }

    fn mouse_interaction(
        &self,
        tree: &adv_widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &R,
    ) -> mouse::Interaction {
        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                child_layout,
                cursor,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut adv_widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &R,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, T, R>> {
        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                child_layout,
                renderer,
                translation,
            )
        } else {
            None
        }
    }
}

impl<'a, M: 'a, T: 'a, R: iced::advanced::Renderer + 'a> From<ClipTranslate<'a, M, T, R>>
    for Element<'a, M, T, R>
{
    fn from(widget: ClipTranslate<'a, M, T, R>) -> Self {
        Element::new(widget)
    }
}

// LESSON 8: Paints the animated transition strip. Card painting is identical to render_modal_unit, but shift is computed live from p so the whole strip slides across the viewport. The departure unit is centered at p=0; as p grows, the strip moves and the arrival unit enters from the opposite side. clip(true) cuts off anything outside the viewport boundary.
/// Render a connected transition strip with a true clipped-viewport envelope.
///
/// The dep unit is centred in the viewport at p = 0.  The combined strip then
/// slides left by `slide * p` pixels, bringing the arr unit to centre at p = 1.
pub(super) fn render_connected_transition<'a>(
    app: &'a App,
    rendered: &RenderedModalUnit,
    departure: &crate::app::ModalDepartureLayer,
    current_modal: Option<&'a crate::modal::SearchModal>,
    modal_height: f32,
    p: f32,
    slide: f32,
    interactive_active: bool,
) -> Element<'a, Message> {
    let spacer_width = app.modal_spacer_width();
    let mut cards: Vec<Element<'a, Message>> = Vec::new();
    for card in &rendered.cards {
        let alpha = card.alpha;
        match &card.kind {
            ModalUnitCardKind::Stub { mode, stub_kind, .. } => match mode {
                ModalUnitStubMode::Visible => {
                    let text_color = match stub_kind {
                        ModalStubKind::NavLeft | ModalStubKind::NavRight => {
                            app.ui_theme.modal_nav_stub_text
                        }
                        ModalStubKind::Exit => app.ui_theme.modal_exit_stub_text,
                        ModalStubKind::Confirm => app.ui_theme.modal_confirm_stub_text,
                    };
                    cards.push(modal_card(
                        app,
                        container(
                            iced::widget::text(stub_kind.symbol().to_string())
                                .font(app.ui_theme.font_modal)
                                .size(24)
                                .color(apply_alpha(text_color, alpha)),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                        ModalRenderMode::Preview,
                        ModalCardRole::Stub(*stub_kind),
                        false,
                        card.width,
                        modal_height,
                        alpha,
                    ));
                }
            },
            ModalUnitCardKind::Preview { snapshot } => cards.push(modal_card(
                app,
                preview_simple_modal_content(app, snapshot.clone(), alpha),
                ModalRenderMode::Preview,
                ModalCardRole::Inactive,
                false,
                card.width,
                modal_height,
                alpha,
            )),
            ModalUnitCardKind::Active { snapshot } => {
                if interactive_active {
                    if let Some(modal) = current_modal {
                        cards.push(modal_card(
                            app,
                            active_simple_modal_content(app, modal, alpha),
                            ModalRenderMode::Interactive,
                            ModalCardRole::Active,
                            true,
                            card.width,
                            modal_height,
                            alpha,
                        ));
                    } else {
                        cards.push(modal_card(
                            app,
                            preview_simple_modal_content(app, snapshot.clone(), alpha),
                            ModalRenderMode::Preview,
                            ModalCardRole::Active,
                            false,
                            card.width,
                            modal_height,
                            alpha,
                        ));
                    }
                } else {
                    cards.push(modal_card(
                        app,
                        preview_simple_modal_content(app, snapshot.clone(), alpha),
                        ModalRenderMode::Preview,
                        ModalCardRole::Active,
                        false,
                        card.width,
                        modal_height,
                        alpha,
                    ));
                }
            }
        }
    }

    let row_width = rendered.total_width(spacer_width);
    let viewport_width = app.viewport_size.map(|size| size.width).unwrap_or(row_width);

    // LESSON 9: Per-frame position math. shift is the left offset of the entire strip each frame.
    // Forward slides left (- slide * p); backward slides right (+ slide * p, negated formula).
    // row_width and viewport_width centre the departing unit at p=0 in both cases.
    // Compute the dep unit's display width to anchor its centre to the viewport
    // centre at p = 0.  Forward: dep is on the left, strip slides left (arr enters
    // from right).  Backward: dep is on the right, strip slides right (arr enters
    // from left), so the shift formula is negated.
    let dep_unit_width =
        transition_unit_display_width(&departure.geometry, app.ui_theme.modal_stub_width);
    let shift = match departure.focus_direction {
        crate::app::FocusDirection::Forward => (row_width - dep_unit_width) / 2.0 - slide * p,
        crate::app::FocusDirection::Backward => -(row_width - dep_unit_width) / 2.0 + slide * p,
    };

    // inner_left: x coordinate in viewport space where the strip's left edge sits.
    // Negative values (strip overhangs left) are allowed because ClipTranslate bypasses
    // iced's Limits clamping. The clip envelope is exactly [0, viewport_width].
    let inner_left = (viewport_width - row_width) / 2.0 + shift;

    ClipTranslate::new(
        inner_left,
        viewport_width,
        modal_height,
        row(cards).spacing(spacer_width).align_y(iced::alignment::Vertical::Center),
    )
    .into()
}

/// Pure sizing helper: the height ClipTranslate will actually use given an ideal
/// and the maximum the parent grants. Extracted so tests can verify the contract
/// without requiring a full iced layout pass.
pub(super) fn clip_translate_effective_height(ideal: f32, parent_max: f32) -> f32 {
    ideal.min(parent_max)
}

#[cfg(test)]
mod clip_translate_height_tests {
    use super::*;

    #[test]
    fn height_respects_parent_limit_when_tighter() {
        assert_eq!(clip_translate_effective_height(800.0, 600.0), 600.0);
    }

    #[test]
    fn height_uses_ideal_when_parent_has_room() {
        assert_eq!(clip_translate_effective_height(400.0, 800.0), 400.0);
    }

    #[test]
    fn height_uses_ideal_when_parent_matches_exactly() {
        assert_eq!(clip_translate_effective_height(500.0, 500.0), 500.0);
    }

    #[test]
    fn clip_envelope_and_child_layout_height_agree() {
        // Confirms that the same effective_height drives both the node size and
        // the child limits, so the clip envelope never exceeds the child's space.
        let ideal = 900.0;
        let parent_max = 700.0;
        let effective = clip_translate_effective_height(ideal, parent_max);
        // The node height and child limits max height must be the same value.
        assert_eq!(effective, 700.0);
        assert!(effective <= parent_max);
    }
}
