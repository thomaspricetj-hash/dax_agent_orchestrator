//! Delta primitives and helpers — MAX‑TIER
//!
//! Provides:
//! - DeltaState trait (object safe)
//! - WeightedDelta helper trait (renamed method to avoid collisions)
//! - ZeroDelta placeholder
//! - Example concrete deltas: SimpleDelta, NumericDelta
//! - Small helpers for downcasting and weight access

use std::any::Any;
use std::fmt::Debug;

/// Object-safe delta trait used throughout the system.
pub trait DeltaState: Send + Sync {
    /// Allow downcasting by reference.
    fn as_any(&self) -> &dyn Any;

    /// Allow downcasting by mutable reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Optional weight accessor. Default: None (unweighted).
    fn weight(&self) -> Option<f32> {
        None
    }

    /// Optional setter for weight. Default: no-op.
    fn set_weight(&mut self, _w: f32) {}

    /// Optional metadata accessor for richer deltas.
    fn metadata(&self) -> Option<DeltaMetadata> {
        None
    }
}

/// Metadata that some deltas may expose.
#[derive(Clone, Debug)]
pub struct DeltaMetadata {
    pub weight: f32,
    pub provenance: Option<String>,
}

impl DeltaMetadata {
    pub fn new(weight: f32) -> Self {
        Self {
            weight,
            provenance: None,
        }
    }
}

/// A small helper trait for types that are inherently weighted.
/// NOTE: method is named `weighted` (not `weight`) to avoid colliding with
/// `DeltaState::weight()` and causing ambiguous method resolution.
pub trait WeightedDelta {
    fn weighted(&self) -> f32;
    fn set_weighted(&mut self, w: f32);
}

/// A zero/empty delta placeholder that implements DeltaState.
/// Useful as a safe return value when no real delta exists.
#[derive(Clone, Debug)]
pub struct ZeroDelta;

impl DeltaState for ZeroDelta {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn weight(&self) -> Option<f32> {
        Some(0.0)
    }

    fn metadata(&self) -> Option<DeltaMetadata> {
        Some(DeltaMetadata::new(0.0))
    }
}

// ============================================================================
// Example concrete delta: SimpleDelta
// ============================================================================

#[derive(Clone, Debug)]
pub struct SimpleDelta {
    pub description: String,
    pub w: f32,
}

impl SimpleDelta {
    pub fn new(description: impl Into<String>, w: f32) -> Self {
        Self {
            description: description.into(),
            w,
        }
    }
}

impl WeightedDelta for SimpleDelta {
    fn weighted(&self) -> f32 {
        self.w
    }

    fn set_weighted(&mut self, w: f32) {
        self.w = w;
    }
}

impl DeltaState for SimpleDelta {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    /// Expose weight via the unified DeltaState accessor so callers can use
    /// `delta.weight()` without ambiguity.
    fn weight(&self) -> Option<f32> {
        Some(self.weighted())
    }

    fn set_weight(&mut self, w: f32) {
        self.set_weighted(w)
    }

    fn metadata(&self) -> Option<DeltaMetadata> {
        Some(DeltaMetadata::new(self.w))
    }
}

// ============================================================================
// Example concrete delta: NumericDelta
// ============================================================================

#[derive(Clone, Debug)]
pub struct NumericDelta {
    pub value: f64,
    pub w: f32,
}

impl NumericDelta {
    pub fn new(value: f64, w: f32) -> Self {
        Self { value, w }
    }
}

impl WeightedDelta for NumericDelta {
    fn weighted(&self) -> f32 {
        self.w
    }

    fn set_weighted(&mut self, w: f32) {
        self.w = w;
    }
}

impl DeltaState for NumericDelta {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn weight(&self) -> Option<f32> {
        Some(self.weighted())
    }

    fn set_weight(&mut self, w: f32) {
        self.set_weighted(w)
    }

    fn metadata(&self) -> Option<DeltaMetadata> {
        Some(DeltaMetadata::new(self.w))
    }
}

// ============================================================================
// Small utility helpers
// ============================================================================

/// Downcast helper: try to get a reference to a concrete delta type.
pub fn delta_downcast_ref<T: 'static>(d: &dyn DeltaState) -> Option<&T> {
    d.as_any().downcast_ref::<T>()
}

/// Downcast helper: try to get a mutable reference to a concrete delta type.
pub fn delta_downcast_mut<T: 'static>(d: &mut dyn DeltaState) -> Option<&mut T> {
    d.as_any_mut().downcast_mut::<T>()
}


