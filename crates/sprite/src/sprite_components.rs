use bevy::math::{Vec2, UVec2};
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::prelude::Spritesheet;
use common::common_components::*;
use common::common_types::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(EntityPrefix::new("SpriteConfig"), AssetScoped,)]
pub struct SpriteConfig;




#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
#[relationship(relationship_target = HeldSprites)]
#[require(EntityPrefix::new("Sprite"), Replicated,)]
pub struct SpriteHolderRef {#[relationship]#[entities]pub base: Entity, }

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = SpriteHolderRef)]
pub struct HeldSprites(Vec<Entity>);
impl HeldSprites {pub fn sprite_ents(&self) -> &Vec<Entity> { &self.0 }}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy )]
pub struct WalkAnim;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy )]
pub struct SwimAnim{pub use_still: bool,}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct FlyAnim{pub use_still: bool,}


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ExcludedFromBaseAnimPickingSystem;


#[derive(Component, Debug, Deserialize, Serialize,  Clone, Copy)]
pub enum FlipHorizIfDir{Left, Right, Any,}





#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct SpriteConfigRef(#[entities] pub Entity);





#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct AtlasLayoutData {pub spritesheet_size: UVec2, pub frame_size: UVec2,}

impl AtlasLayoutData {
    pub fn new(spritesheet_size: [u32; 2], frame_size: [u32; 2]) -> Self {
        Self { spritesheet_size: spritesheet_size.into(), frame_size: frame_size.into(), }
    }
}
impl AtlasLayoutData {
    pub fn into_texture_atlas(
        self,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> TextureAtlas {
        let spritesheet_size = self.spritesheet_size;
        let frame_size = self.frame_size;
        TextureAtlas {
            layout: atlas_layouts.add(
                Spritesheet::new(
                    spritesheet_size.y as usize,
                    spritesheet_size.x as usize,
                )
                .atlas_layout(frame_size.x, frame_size.y)
            ),
            ..Default::default()
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct ColorHolder(pub Color);//NO HACER PARTE DE SpriteDataBundle
/// Trait for 2D scale components.
pub trait Scale2DComponent: Sized {
    fn new(scale: Vec2) -> Self;
    fn from_vec2(v: Vec2) -> Self { Self::new(v) }
    fn from_array(v: [f32; 2]) -> Self { Self::new(Vec2::from(v)) }
    fn as_vec2(&self) -> Vec2;
}

/// Macro to implement a strongly-typed 2D scale component and its ops.
macro_rules! define_scale2d_type {
    ($name:ident) => {
        #[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Reflect)]
        pub struct $name(pub Vec2);

        impl $name {
            pub fn new(scale: Vec2) -> Self {
                let mut fixed = scale;
                let mut warned = false;
                if fixed.x <= 0.0 { fixed.x = 1.0; warned = true; }
                if fixed.y <= 0.0 { fixed.y = 1.0; warned = true; }
                if warned {
                    warn!("Non-positive scale component detected in {}::new({:?}), set to 1.0", stringify!($name), scale);
                }
                Self(fixed)
            }
            pub fn set(&mut self, scale: Vec2) { *self = Self::new(scale); }
            pub fn as_vec2(&self) -> Vec2 { self.0 }
        }
        impl Default for $name {fn default() -> Self {Self::new(Vec2::ONE)}}

        impl Scale2DComponent for $name {
            fn new(scale: Vec2) -> Self { Self::new(scale) }
            fn as_vec2(&self) -> Vec2 { self.0 }
        }
        impl From<Vec2> for $name { fn from(v: Vec2) -> Self { Self::new(v) } }
        impl From<[f32; 2]> for $name { fn from(v: [f32; 2]) -> Self { Self::new(Vec2::from(v)) } }
        impl std::ops::Mul for $name {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self { Self(self.0 * rhs.0) }
        }
        impl std::ops::MulAssign for $name {
            fn mul_assign(&mut self, rhs: Self) { self.0 *= rhs.0; }
        }
    };
}
define_scale2d_type!(Scale2D);
define_scale2d_type!(ScaleLookUp);
define_scale2d_type!(ScaleLookDown);
define_scale2d_type!(ScaleLookUpDown);
define_scale2d_type!(ScaleSideways);

macro_rules! impl_cross_mul {
    ($A:ty, $B:ty) => {
        impl std::ops::Mul<$B> for $A {
            type Output = $A;
            fn mul(self, rhs: $B) -> $A { <$A>::new(self.as_vec2() * rhs.as_vec2()) }
        }
        impl std::ops::Mul<$A> for $B {
            type Output = $B;
            fn mul(self, rhs: $A) -> $B { <$B>::new(self.as_vec2() * rhs.as_vec2()) }
        }
        impl std::ops::MulAssign<$B> for $A {
            fn mul_assign(&mut self, rhs: $B) { self.0 *= rhs.as_vec2(); }
        }
        impl std::ops::MulAssign<$A> for $B {
            fn mul_assign(&mut self, rhs: $A) { self.0 *= rhs.as_vec2(); }
        }
    };
}

// Cross-multiplication for all pairs (excluding reflexive, already covered)
impl_cross_mul!(Scale2D, ScaleLookUp);
impl_cross_mul!(Scale2D, ScaleLookDown);
impl_cross_mul!(Scale2D, ScaleLookUpDown);
impl_cross_mul!(Scale2D, ScaleSideways);
impl_cross_mul!(ScaleLookUp, ScaleLookDown);
impl_cross_mul!(ScaleLookUp, ScaleLookUpDown);
impl_cross_mul!(ScaleLookUp, ScaleSideways);
impl_cross_mul!(ScaleLookDown, ScaleLookUpDown);
impl_cross_mul!(ScaleLookDown, ScaleSideways);
impl_cross_mul!(ScaleLookUpDown, ScaleSideways);

macro_rules! define_offset2d_type {
    ($name:ident) => {
        #[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
        pub struct $name(pub Vec2);
        impl From<Vec2> for $name { fn from(v: Vec2) -> Self { $name(v) } }
        impl From<[f32; 2]> for $name { fn from(v: [f32; 2]) -> Self { $name(Vec2::from(v)) } }
        impl std::ops::Add for $name { type Output = Self; fn add(self, rhs: Self) -> Self { $name(self.0 + rhs.0) } }
        impl std::ops::AddAssign for $name { fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; } }
    };
}

define_offset2d_type!(Offset2D);
define_offset2d_type!(OffsetUpDown);
define_offset2d_type!(OffsetDown);
define_offset2d_type!(OffsetUp);
define_offset2d_type!(OffsetSideways);

macro_rules! impl_cross_sum {
    ($A:ty, $B:ty) => {
        impl std::ops::Add<$B> for $A {
            type Output = $A;
            fn add(self, rhs: $B) -> $A { <$A>::from(self.0 + rhs.0) }
        }
        impl std::ops::Add<$A> for $B {
            type Output = $B;
            fn add(self, rhs: $A) -> $B { <$B>::from(self.0 + rhs.0) }
        }
        impl std::ops::AddAssign<$B> for $A {
            fn add_assign(&mut self, rhs: $B) { self.0 += rhs.0; }
        }
        impl std::ops::AddAssign<$A> for $B {
            fn add_assign(&mut self, rhs: $A) { self.0 += rhs.0; }
        }
    };
}

// Cross-sum for all pairs (excluding reflexive, already covered)
impl_cross_sum!(Offset2D, OffsetUpDown);
impl_cross_sum!(Offset2D, OffsetDown);
impl_cross_sum!(Offset2D, OffsetUp);
impl_cross_sum!(Offset2D, OffsetSideways);
impl_cross_sum!(OffsetUpDown, OffsetDown);
impl_cross_sum!(OffsetUpDown, OffsetUp);
impl_cross_sum!(OffsetUpDown, OffsetSideways);
impl_cross_sum!(OffsetDown, OffsetUp);
impl_cross_sum!(OffsetDown, OffsetSideways);
impl_cross_sum!(OffsetUp, OffsetSideways);



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct OffsetForChildren(pub HashMap<Category, Offset2D>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Category(u64);

impl Category {
    pub fn new<S: Into<String>>(id: S) -> Self {
        let id_str = id.into();
        let mut hasher = DefaultHasher::new();
        id_str.hash(&mut hasher);
        Self(hasher.finish())
    }
}
impl std::fmt::Display for Category {fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "Category({})", self.0)}}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct Categories(pub HashSet<Category>);

impl Categories {
    pub fn new<S: Into<String>>(ids: impl IntoIterator<Item = S>) -> Self {
        let mut set = HashSet::new();
        for id in ids {
            let id_str = id.into();
            let mut hasher = DefaultHasher::new();
            id_str.hash(&mut hasher);
            set.insert(Category(hasher.finish()));
        }
        Self(set)
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct Exclusive;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct BecomeChildOfSpriteWithCategory (pub Category);
impl BecomeChildOfSpriteWithCategory {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(Category::new(id))
    }
    pub fn category(&self) -> &Category {&self.0}
}




// NO USAR ESTOS DOS PARA BEINGS
#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(Replicated)]
pub struct SpriteConfigStringIds(pub Vec<String>);
impl SpriteConfigStringIds {
    pub fn new<S: Into<String>>(ids: impl IntoIterator<Item = S>) -> Self {
        Self(ids.into_iter().map(|s| s.into()).collect())
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone )]
pub struct SpriteCfgsToBuild(#[entities] pub HashSet<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone )]
pub struct SpriteCfgsBuiltSoFar(#[entities] pub HashSet<Entity>);

