#[allow(unused_imports)] use bevy::prelude::*;
use crate::{
    common_components::*,
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
};
use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::{Index, IndexMut}};

pub type HashIdToEntityMap = HashIdMap<Entity>;

#[derive(Component, Clone, Debug, Deserialize, Serialize, Default)]
pub struct BiArray<T> {
    width: usize,
    height: usize,
    values: Vec<T>,
}
impl<T> BiArray<T> {
    pub fn from_vec(width: usize, height: usize, values: Vec<T>) -> Self {
        assert_eq!(values.len(), width.saturating_mul(height));
        Self {
            width,
            height,
            values,
        }
    }
    pub fn from_fn(width: usize, height: usize, mut f: impl FnMut(usize, usize) -> T) -> Self {
        let mut values = Vec::with_capacity(width.saturating_mul(height));
        for y in 0..height {
            for x in 0..width {
                values.push(f(x, y));
            }
        }
        Self::from_vec(width, height, values)
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn flat_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.values.get(self.flat_index(x, y))
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let i = self.flat_index(x, y);
        self.values.get_mut(i)
    }
    pub fn values(&self) -> &[T] {
        self.values.as_slice()
    }
    pub fn values_mut(&mut self) -> &mut [T] {
        self.values.as_mut_slice()
    }
}
impl<T> Index<(usize, usize)> for BiArray<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.values[self.flat_index(index.0, index.1)]
    }
}
impl<T> IndexMut<(usize, usize)> for BiArray<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let i = self.flat_index(index.0, index.1);
        &mut self.values[i]
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Reflect)]
pub struct FixedStr<const N: usize>([u8; N]);

impl<const N: usize> InspectorPrimitive for FixedStr<N> {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        let mut s = self.as_str().to_string();
        let mut changed = false;
        if ui.text_edit_singleline(&mut s).changed() {
            if let Ok(fixed) = FixedStr::new_with_result(&s, 0) {
                *self = fixed;
                changed = true;
            }
        }
        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        ui.label(self.as_str());
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringLengthError {
    TooShort(String, u8),
    TooLong(String, u8),
}

impl Into<BevyError> for StringLengthError {
    fn into(self) -> BevyError {
        BevyError::from(self.to_string())
    }
}

impl Display for StringLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringLengthError::TooShort(s, min) => write!(
                f,
                "String '{}'(len={}) is too short by {} characters (min len: {})",
                s,
                s.len(),
                min.saturating_sub(s.len() as u8),
                min
            ),
            StringLengthError::TooLong(s, max) => write!(
                f,
                "String '{}'(len={}) is too long by {} characters (max len: {})",
                s,
                s.len(),
                s.len().saturating_sub(*max as usize),
                max
            ),
        }
    }
}

impl<const N: usize> FixedStr<N> {
    pub fn trunc<S: AsRef<str>>(s: S) -> Self {
        let bytes = s.as_ref().as_bytes();
        let mut arr = [0u8; N];
        let len = bytes.len().min(N);
        let start = bytes.len().saturating_sub(len);
        arr[..len].copy_from_slice(&bytes[start..]);
        Self(arr)
    }

    pub fn new_with_result<S: AsRef<str>>(s: S, min_length: u8) -> Result<Self, StringLengthError> {
        let len = s.as_ref().len();
        if len < min_length as usize {
            return Err(StringLengthError::TooShort(s.as_ref().to_string(), min_length));
        }
        if len > N {
            return Err(StringLengthError::TooLong(s.as_ref().to_string(), N as u8));
        }
        Ok(Self::trunc(s))
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    pub fn as_str(&self) -> &str {
        let nul_pos = self.0.iter().position(|&b| b == 0).unwrap_or(N);
        std::str::from_utf8(&self.0[..nul_pos]).unwrap_or("")
    }
}


impl<const N: usize> Default for FixedStr<N> {fn default() -> Self { Self([0u8; N]) } }
impl<const N: usize> std::fmt::Display for FixedStr<N> { #[inline(always)] fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { std::fmt::Display::fmt(self.as_str(), f) } }
impl<const N: usize> std::fmt::Debug for FixedStr<N> { #[inline(always)] fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.as_str()) } }
impl<const N: usize> serde::Serialize for FixedStr<N> { fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer, { serializer.serialize_str(self.as_str()) } }
impl<'de, const N: usize> serde::Deserialize<'de> for FixedStr<N> { fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de>, { let s = <&str>::deserialize(deserializer)?; Ok(FixedStr::trunc(s)) } }
impl<const N: usize> From<&str> for FixedStr<N> { fn from(s: &str) -> Self { FixedStr::trunc(s) } }
impl<const N: usize> From<String> for FixedStr<N> { fn from(s: String) -> Self { FixedStr::trunc(s) } }
impl<const N: usize> AsRef<str> for FixedStr<N> { fn as_ref(&self) -> &str { self.as_str() } }
