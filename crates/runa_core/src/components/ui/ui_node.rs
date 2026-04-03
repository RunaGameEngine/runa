use glam::Vec2;
use runa_asset::{Handle, TextureAsset};

type UiColor = [f32; 4];

#[derive(Clone, Copy, Debug, Default)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl UiRect {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct UiNodeId(pub u32);

pub struct UiNode {
    pub id: UiNodeId,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub kind: UiNodeKind,
    pub layout: LayoutProps,
    pub style: StyleProps,
    pub computed: ComputedLayout,
    pub visible: bool,
}

impl UiNode {
    pub fn new(id: UiNodeId, parent: Option<UiNodeId>, kind: UiNodeKind) -> Self {
        Self {
            id,
            parent,
            children: Vec::new(),
            kind,
            layout: LayoutProps::default(),
            style: StyleProps::default(),
            computed: ComputedLayout::default(),
            visible: true,
        }
    }
}

pub enum UiNodeKind {
    Container(ContainerKind),
    Image(ImageProps),
    Text(TextProps),
}

pub enum ContainerKind {
    Free,
    HorizontalBox,
    VerticalBox,
}

#[derive(Clone, Copy, Debug)]
pub struct LayoutProps {
    pub anchor: Anchor,
    pub position: Vec2,
    // pub size: UiSize,
    pub min_size: Vec2,
    pub max_size: Vec2,
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,
    pub gap: f32,
}

impl Default for LayoutProps {
    fn default() -> Self {
        Self {
            anchor: Anchor::TopLeft,
            position: glam::Vec2::ZERO,
            min_size: glam::Vec2::ZERO,
            max_size: glam::Vec2::new(f32::INFINITY, f32::INFINITY),
            margin: EdgeInsets::ZERO,
            padding: EdgeInsets::ZERO,
            gap: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StyleProps {
    pub background: Option<UiColor>,
    pub opacity: f32,
    pub z_index: i16,
}

impl Default for StyleProps {
    fn default() -> Self {
        Self {
            background: None,
            opacity: 1.0,
            z_index: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ComputedLayout {
    pub rect: UiRect,
    pub content_rect: UiRect,
}

#[derive(Clone, Copy, Debug)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Stretch,
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl EdgeInsets {
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };
}

pub struct TextProps {
    pub text: String,
    pub font: Option<FontId>,
    pub font_size: u16,
    pub color: [f32; 4],
    pub line_height: Option<f32>,
    pub align: TextAlign,
}

pub struct ImageProps {
    pub texture: Option<Handle<TextureAsset>>,
    pub tint: [f32; 4],
    pub uv: [f32; 4],
}

pub enum TextAlign {}

pub struct FontId {}
