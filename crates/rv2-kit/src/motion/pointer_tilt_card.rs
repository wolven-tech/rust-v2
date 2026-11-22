//! Pointer tilt card: the card leans toward the pointer, a glare follows it, and the
//! card settles back to flat when the pointer leaves.
//!
//! Design choice: one `document::eval` per card installs the pointer listeners. They
//! write two unit-range variables (`--kit-tilt-x`, `--kit-tilt-y`); the angle, the
//! glare position and the transitions are CSS, so no per-move re-render happens.
//! Design choice: the pointer-side edge comes toward the viewer. `tilt_degrees`
//! is the reference for that sign and is what the CSS calc implements.
//! Design choice: the glare centre is the pointer mirrored through the card centre,
//! so the highlight sits opposite the lean and still moves with the pointer.
//! Design choice: the focus cue is one lean toward the upper edge, then a settle.
//! The spec's "same lean as a centred hover" is flat in its own acceptance check,
//! so it is not reproduced literally.
//!
//! Browser checks: a corner lean and a centre return to flat; leaving springs back;
//! the glare follows the pointer; interactive children still take clicks and focus;
//! reduced motion stays flat; `TouchBehaviour::None` ignores a tap; phone width keeps
//! the layout.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/pointer-tilt-card.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TiltStrength {
    Subtle,
    #[default]
    Standard,
    Strong,
}

impl TiltStrength {
    fn as_str(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Standard => "standard",
            Self::Strong => "strong",
        }
    }

    /// Largest lean, in degrees, at a corner.
    pub fn degrees(self) -> f64 {
        match self {
            Self::Subtle => 4.0,
            Self::Standard => 8.0,
            Self::Strong => 12.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TouchBehaviour {
    #[default]
    None,
    TapToTilt,
}

impl TouchBehaviour {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::TapToTilt => "tap_to_tilt",
        }
    }
}

/// Pointer position within the card as a pair in `-1..=1`, centre at `(0, 0)`.
/// A card with no size reports the centre.
pub fn unit_pointer(x: f64, y: f64, width: f64, height: f64) -> (f64, f64) {
    if width <= 0.0 || height <= 0.0 {
        return (0.0, 0.0);
    }
    let ux = ((x / width) * 2.0 - 1.0).clamp(-1.0, 1.0);
    let uy = ((y / height) * 2.0 - 1.0).clamp(-1.0, 1.0);
    (ux, uy)
}

/// `(rotate_x, rotate_y)` in degrees. A pointer at the top edge gives a negative
/// `rotate_x`, which brings the top edge toward the viewer.
pub fn tilt_degrees(strength: TiltStrength, unit: (f64, f64)) -> (f64, f64) {
    let max = strength.degrees();
    (unit.1 * max, -unit.0 * max)
}

/// Glare centre in percent of the card, mirrored through the centre from the pointer.
pub fn glare_percent(unit: (f64, f64)) -> (f64, f64) {
    (50.0 - unit.0 * 50.0, 50.0 - unit.1 * 50.0)
}

#[component]
pub fn PointerTiltCard(
    children: Element,
    #[props(default)] max_tilt: TiltStrength,
    #[props(default = true)] glare: bool,
    #[props(default)] touch_behaviour: TouchBehaviour,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("kit-tilt-{}", next_id()));

    let listen_id = id.clone();
    use_effect(move || {
        let script = format!(
            "const el = document.getElementById({listen_id:?});
             if (!el || el.dataset.bound) return true;
             el.dataset.bound = '1';
             const reduced = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;
             const unit = (e) => {{
               const r = el.getBoundingClientRect();
               if (!r.width || !r.height) return [0, 0];
               const ux = Math.max(-1, Math.min(1, ((e.clientX - r.left) / r.width) * 2 - 1));
               const uy = Math.max(-1, Math.min(1, ((e.clientY - r.top) / r.height) * 2 - 1));
               return [ux, uy];
             }};
             const set = (ux, uy) => {{
               el.style.setProperty('--kit-tilt-x', ux);
               el.style.setProperty('--kit-tilt-y', uy);
             }};
             const clear = () => {{ el.removeAttribute('data-active'); set(0, 0); }};
             const lean = (e) => {{
               const [ux, uy] = unit(e);
               set(ux, uy);
               el.setAttribute('data-active', 'true');
             }};
             el.addEventListener('pointermove', (e) => {{
               if (e.pointerType === 'touch' || reduced()) return;
               lean(e);
             }});
             el.addEventListener('pointerleave', (e) => {{
               if (e.pointerType !== 'touch') clear();
             }});
             el.addEventListener('pointerdown', (e) => {{
               if (e.pointerType !== 'touch' || el.dataset.touch !== 'tap_to_tilt') return;
               if (el.hasAttribute('data-active')) {{ clear(); return; }}
               lean(e);
             }});
             const outside = (e) => {{
               if (!el.isConnected) {{ document.removeEventListener('pointerdown', outside, true); return; }}
               if (el.hasAttribute('data-active') && !el.contains(e.target)) clear();
             }};
             document.addEventListener('pointerdown', outside, true);
             el.addEventListener('focusin', (e) => {{
               if (e.target === el || reduced() || !e.target.matches(':focus-visible')) return;
               el.removeAttribute('data-nudge');
               void el.offsetWidth;
               el.setAttribute('data-nudge', '');
             }});
             el.addEventListener('animationend', (e) => {{
               if (e.target === el) el.removeAttribute('data-nudge');
             }});
             return true;"
        );
        spawn(async move {
            let _ = document::eval(&script).await;
        });
    });

    let class = class_list("kit-tilt", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{id}", class,
            "data-strength": max_tilt.as_str(),
            "data-touch": touch_behaviour.as_str(),
            div { class: "kit-tilt__content", {children} }
            if glare {
                div { class: "kit-tilt__glare", "aria-hidden": "true" }
            }
        }
    }
}
