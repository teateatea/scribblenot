// Transition data types: the frozen state captured at modal transition start.
// These live here rather than in app.rs to keep that file readable.
// All types are re-exported from app.rs, so external code uses crate::app::* as before.

use crate::modal::SearchModal;
use crate::modal_layout::{
    modal_list_view_dimensions, ModalListViewSnapshot, ModalStubKind, SimpleModalUnitLayout,
};
use std::collections::HashMap;
use std::time::Instant;

// LESSON 1: Labels which way the user navigated (Forward or Backward). The strip slides
// opposite to focus: moving forward means the strip slides left.
/// Direction focus moved to trigger a transition.
/// Strip slides in the opposite direction: Forward = strip moves left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Forward,  // focus moved to a higher-index unit (rightward)
    Backward, // focus moved to a lower-index unit (leftward)
}

// LESSON 2: A menu of animation curve options. The apply() function shapes raw 0-to-1 progress
// into a curve (e.g. fast-then-slow) so the motion doesn't look mechanical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalTransitionEasing {
    Linear,
    QuadInOut,
    CubicInOut,
    SineInOut,
    ExpoIn,
    ExpoInOut,
    ExpoOut,
}

impl ModalTransitionEasing {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Self::Linear => simple_easing::linear(t),
            Self::QuadInOut => simple_easing::quad_in_out(t),
            Self::CubicInOut => simple_easing::cubic_in_out(t),
            Self::SineInOut => simple_easing::sine_in_out(t),
            Self::ExpoIn => simple_easing::expo_in(t),
            Self::ExpoInOut => simple_easing::expo_in_out(t),
            Self::ExpoOut => simple_easing::expo_out(t),
        }
    }
}

// LESSON 3: A frozen snapshot of a unit's physical dimensions (card widths, positions, stub kinds).
// Captured at transition start so a window resize mid-animation can't jitter the in-flight layers.
/// Frozen geometry for one unit, captured at transition start.
/// Insulates in-flight layers from layout rebuilds (e.g. window resize).
#[derive(Debug, Clone)]
#[allow(dead_code)] // Part 3 fields: unit_index, modal_index_range, modal_x_offsets, first/last_list_id
pub struct UnitGeometry {
    pub unit_index: usize,
    pub modal_index_range: std::ops::Range<usize>,
    pub shows_stubs: bool,
    pub leading_stub_kind: Option<ModalStubKind>,
    pub trailing_stub_kind: Option<ModalStubKind>,
    pub effective_spacer_width: f32,
    pub modal_widths: Vec<f32>,
    pub modal_x_offsets: Vec<f32>,
    /// HierarchyList.id of the leftmost list in this unit. Required by Part 3.
    pub first_list_id: String,
    /// HierarchyList.id of the rightmost list in this unit. Required by Part 3.
    pub last_list_id: String,
}

impl UnitGeometry {
    /// Build frozen geometry for a unit from the given layout, or return None if the
    /// unit index is out of range.
    pub fn from_layout(
        layout: &SimpleModalUnitLayout,
        unit_index: usize,
        modal: &SearchModal,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        effective_spacer_width: f32,
    ) -> Option<Self> {
        let unit = layout.units.get(unit_index)?;
        let n = layout.sequence.snapshots.len();
        let semantics = modal.edge_semantics(assigned_values, sticky_values);

        let leading_stub_kind = if unit.shows_stubs {
            Some(if unit.start == 0 {
                semantics.left
            } else {
                ModalStubKind::NavLeft
            })
        } else {
            None
        };
        let trailing_stub_kind = if unit.shows_stubs {
            Some(if unit.end + 1 >= n {
                semantics.right
            } else {
                ModalStubKind::NavRight
            })
        } else {
            None
        };

        let modal_widths: Vec<f32> = (unit.start..=unit.end)
            .map(|i| {
                layout
                    .sequence
                    .snapshots
                    .get(i)
                    .map(|s| modal_list_view_dimensions(s).0)
                    .unwrap_or(0.0)
            })
            .collect();

        let mut modal_x_offsets = Vec::with_capacity(modal_widths.len());
        let mut x = 0.0f32;
        for (i, &w) in modal_widths.iter().enumerate() {
            modal_x_offsets.push(x);
            if i + 1 < modal_widths.len() {
                x += w + effective_spacer_width;
            }
        }

        let first_list_id = modal
            .field_flow
            .lists
            .get(unit.start)
            .map(|l| l.id.clone())
            .unwrap_or_default();
        let last_list_id = modal
            .field_flow
            .lists
            .get(unit.end)
            .map(|l| l.id.clone())
            .unwrap_or_default();

        Some(Self {
            unit_index,
            modal_index_range: unit.start..unit.end + 1,
            shows_stubs: unit.shows_stubs,
            leading_stub_kind,
            trailing_stub_kind,
            effective_spacer_width,
            modal_widths,
            modal_x_offsets,
            first_list_id,
            last_list_id,
        })
    }
}

// LESSON 4: A frozen copy of what the departing unit visually shows. Kept frozen so that
// user typing in the arriving unit (or any state change) can't alter the departing card mid-animation.
/// Frozen render inputs for one unit's modals, captured at transition start.
/// Prevents the departing unit's visuals from mutating if the user types during a
/// transition. Focus has left this unit, so its render state should not change.
#[derive(Debug, Clone)]
pub struct UnitContentSnapshot {
    pub modals: Vec<ModalListViewSnapshot>,
}

impl UnitContentSnapshot {
    pub fn from_layout(layout: &SimpleModalUnitLayout, unit_index: usize) -> Option<Self> {
        let unit = layout.units.get(unit_index)?;
        let modals: Vec<ModalListViewSnapshot> = layout
            .sequence
            .snapshots
            .get(unit.start..=unit.end)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        Some(Self { modals })
    }

    pub fn from_layout_with_active_override(
        layout: &SimpleModalUnitLayout,
        unit_index: usize,
        active_snapshot: Option<ModalListViewSnapshot>,
    ) -> Option<Self> {
        let unit = layout.units.get(unit_index)?;
        let mut snapshot = Self::from_layout(layout, unit_index)?;
        if let Some(active_snapshot) = active_snapshot {
            let active_index = layout.sequence.active_sequence_index;
            if (unit.start..=unit.end).contains(&active_index) {
                snapshot.modals[active_index - unit.start] = active_snapshot;
            }
        }
        Some(snapshot)
    }
}

// LESSON 5: The clock and config for the incoming unit's animation. Owns started_at/duration/easing
// so the renderer can ask "how far along?" each frame. Content is live (focus is here now).
/// One unit currently sliding and fading in.
/// Uses live modal content (focus is here; user input is correct).
/// Geometry is frozen so resize does not invalidate this layer.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Part 3 fields: unit_index, geometry
pub struct ModalArrivalLayer {
    pub unit_index: usize,
    pub geometry: UnitGeometry,
    pub focus_direction: FocusDirection,
    pub started_at: Instant,
    pub duration_ms: u64,
    pub easing: ModalTransitionEasing,
}

impl ModalArrivalLayer {
    pub fn progress(&self) -> f32 {
        let duration_secs = (self.duration_ms.max(1) as f32) / 1000.0;
        (self.started_at.elapsed().as_secs_f32() / duration_secs).clamp(0.0, 1.0)
    }

    pub fn eased_progress(&self) -> f32 {
        self.easing.apply(self.progress())
    }

    pub fn is_finished(&self) -> bool {
        self.progress() >= 1.0
    }
}

// LESSON 6: The departing unit's animation data. Mirrors ModalArrivalLayer but content and
// geometry are both frozen - focus has left, so nothing here should change mid-animation.
/// One unit currently sliding and fading out.
/// Content and geometry are both frozen at transition start.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Part 3 fields: geometry, started_at, duration_ms, easing
pub struct ModalDepartureLayer {
    pub content: UnitContentSnapshot,
    pub geometry: UnitGeometry,
    pub modal: Option<SearchModal>,
    pub focus_direction: FocusDirection,
    pub started_at: Instant,
    pub duration_ms: u64,
    pub easing: ModalTransitionEasing,
}

#[allow(dead_code)] // Part 3: progress/eased_progress used when departure has its own timing
impl ModalDepartureLayer {
    pub fn progress(&self) -> f32 {
        let duration_secs = (self.duration_ms.max(1) as f32) / 1000.0;
        (self.started_at.elapsed().as_secs_f32() / duration_secs).clamp(0.0, 1.0)
    }

    pub fn eased_progress(&self) -> f32 {
        self.easing.apply(self.progress())
    }

    pub fn is_finished(&self) -> bool {
        self.progress() >= 1.0
    }
}

#[derive(Debug, Clone)]
pub struct ModalCompositionLayer {
    pub modal: SearchModal,
    pub focus_direction: FocusDirection,
    pub started_at: Instant,
    pub duration_ms: u64,
    pub easing: ModalTransitionEasing,
}

impl ModalCompositionLayer {
    pub fn progress(&self) -> f32 {
        let duration_secs = (self.duration_ms.max(1) as f32) / 1000.0;
        (self.started_at.elapsed().as_secs_f32() / duration_secs).clamp(0.0, 1.0)
    }

    pub fn eased_progress(&self) -> f32 {
        self.easing.apply(self.progress())
    }

    pub fn is_finished(&self) -> bool {
        self.progress() >= 1.0
    }
}

#[derive(Debug, Clone)]
pub enum ModalCompositionTransition {
    Open {
        arrival: ModalCompositionLayer,
        slide_distance: f32,
    },
    Close {
        departure: ModalCompositionLayer,
        slide_distance: f32,
    },
}

// LESSON 7: The envelope that pairs arrival + departure + slide_distance into one animation entry.
// The renderer pulls one of these off the list and draws both layers from it.
/// A single animation entry.
#[derive(Debug, Clone)]
pub enum ModalTransitionLayer {
    /// Normal transition: dep and arr form one rigid strip with a shared transition stub.
    /// Both layers share the same x_offset, driven by the arrival's progress.
    /// slide_distance is precomputed at creation from the viewport dimensions that exist
    /// at transition start and stored here so render is independent of live viewport values.
    ConnectedTransition {
        arrival: ModalArrivalLayer,
        departure: ModalDepartureLayer,
        slide_distance: f32,
    },
    ModalOpen {
        arrival: ModalArrivalLayer,
        slide_distance: f32,
    },
    ModalClose {
        departure: ModalDepartureLayer,
        slide_distance: f32,
    },
    // Part 3 queue drain hook: fire next queued transition from here when arrival completes.
}

// LESSON 8: Computes total pixel width of a unit from frozen geometry. Used by app.rs to
// calculate slide_distance before creating the transition layers.
/// Returns the total rendered width (in pixels) of a modal unit from its frozen geometry.
/// This is the sum of stub widths (if any), modal card widths, and inter-element spacers.
pub fn unit_display_width(geometry: &UnitGeometry, stub_width: f32) -> f32 {
    let n = geometry.modal_widths.len();
    let modals: f32 = geometry.modal_widths.iter().sum();
    if geometry.shows_stubs {
        2.0 * stub_width + modals + (n as f32 + 1.0) * geometry.effective_spacer_width
    } else {
        modals + (n.saturating_sub(1) as f32) * geometry.effective_spacer_width
    }
}
