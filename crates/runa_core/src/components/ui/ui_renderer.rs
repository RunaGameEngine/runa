use std::sync::Arc;

use glam::Vec2;
use runa_render_api::{command::UiRect, RenderQueue};

use crate::components::ui::{
    Anchor, ContainerKind, ImageProps, LayoutProps, StyleProps, TextProps, UiNode, UiNodeId,
    UiNodeKind,
};

pub struct UiRenderer {
    pub root_node_path: Option<String>,
    pub ui_asset_path: Option<String>,
    pub space: CanvasSpace,
    pub nodes: Vec<UiNode>,
    pub root: UiNodeId,
    pub dirty_layout: bool,
}

impl UiRenderer {
    pub fn new(space: CanvasSpace) -> Self {
        let root = UiNodeId(0);
        let root_node = UiNode::new(root, None, UiNodeKind::Container(ContainerKind::Free))
            .named("root");

        Self {
            root_node_path: None,
            ui_asset_path: None,
            space,
            nodes: vec![root_node],
            root,
            dirty_layout: true,
        }
    }
    pub fn clear(&mut self) {
        let root_node = UiNode::new(self.root, None, UiNodeKind::Container(ContainerKind::Free))
            .named("root");
        self.nodes.clear();
        self.nodes.push(root_node);
        self.dirty_layout = true;
    }

    pub fn root(&self) -> UiNodeId {
        self.root
    }

    // ── Builder API ──────────────────────────────────────────────

    pub fn container(&mut self, parent: UiNodeId) -> UiNodeBuilder<'_> {
        let id = self.add_node(parent, UiNodeKind::Container(ContainerKind::Free));
        UiNodeBuilder::new(self, id)
    }

    pub fn hbox(&mut self, parent: UiNodeId) -> UiNodeBuilder<'_> {
        let id = self.add_node(parent, UiNodeKind::Container(ContainerKind::HorizontalBox));
        UiNodeBuilder::new(self, id)
    }

    pub fn vbox(&mut self, parent: UiNodeId) -> UiNodeBuilder<'_> {
        let id = self.add_node(parent, UiNodeKind::Container(ContainerKind::VerticalBox));
        UiNodeBuilder::new(self, id)
    }

    pub fn text(&mut self, parent: UiNodeId, content: impl Into<String>) -> UiNodeBuilder<'_> {
        let props = TextProps {
            text: content.into(),
            font: None,
            font_size: 16,
            color: [1.0, 1.0, 1.0, 1.0],
            line_height: None,
            align: crate::components::ui::TextAlign::Left,
        };
        let id = self.add_node(parent, UiNodeKind::Text(props));
        UiNodeBuilder::new(self, id)
    }

    pub fn image(&mut self, parent: UiNodeId) -> UiNodeBuilder<'_> {
        let props = ImageProps {
            texture: None,
            tint: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0, 1.0, 1.0],
        };
        let id = self.add_node(parent, UiNodeKind::Image(props));
        UiNodeBuilder::new(self, id)
    }

    /// Creates a clickable button (container + optional text + optional image)
    pub fn button(
        &mut self,
        parent: UiNodeId,
        label: Option<impl Into<String>>,
        on_click: Option<Box<dyn FnMut() + Send>>,
    ) -> UiNodeBuilder<'_> {
        let btn_id = self.add_node(parent, UiNodeKind::Container(ContainerKind::Free));

        if let Some(text) = label {
            let text_id = self.add_node(
                btn_id,
                UiNodeKind::Text(TextProps {
                    text: text.into(),
                    font: None,
                    font_size: 16,
                    color: [1.0, 1.0, 1.0, 1.0],
                    line_height: None,
                    align: crate::components::ui::TextAlign::Center,
                }),
            );
            if let Some(node) = self.node_mut(text_id) {
                node.layout.anchor = Anchor::Center;
                node.layout.min_size = Vec2::new(0.0, 0.0);
            }
        }

        UiNodeBuilder::new_with_click(self, btn_id, on_click)
    }

    // ── Node lookup ──────────────────────────────────────────────

    /// Find a node by name, searching recursively
    pub fn find_by_name(&self, name: &str) -> Option<UiNodeId> {
        self.find_by_name_recursive(self.root, name)
    }

    fn find_by_name_recursive(&self, id: UiNodeId, name: &str) -> Option<UiNodeId> {
        if let Some(node) = self.node(id) {
            if node.name == name {
                return Some(id);
            }
            for child in &node.children {
                if let Some(found) = self.find_by_name_recursive(*child, name) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Find nodes by name, returns all matches
    pub fn find_all_by_name(&self, name: &str) -> Vec<UiNodeId> {
        let mut results = Vec::new();
        self.find_all_by_name_recursive(self.root, name, &mut results);
        results
    }

    fn find_all_by_name_recursive(&self, id: UiNodeId, name: &str, results: &mut Vec<UiNodeId>) {
        if let Some(node) = self.node(id) {
            if node.name == name {
                results.push(id);
            }
            for child in &node.children {
                self.find_all_by_name_recursive(*child, name, results);
            }
        }
    }

    // ── Legacy helpers ───────────────────────────────────────────

    pub fn add_container(&mut self, parent: UiNodeId, kind: ContainerKind) -> UiNodeId {
        self.add_node(parent, UiNodeKind::Container(kind))
    }

    pub fn add_text(&mut self, parent: UiNodeId, props: TextProps) -> UiNodeId {
        self.add_node(parent, UiNodeKind::Text(props))
    }

    pub fn add_image(&mut self, parent: UiNodeId, props: ImageProps) -> UiNodeId {
        self.add_node(parent, UiNodeKind::Image(props))
    }

    fn add_node(&mut self, parent: UiNodeId, kind: UiNodeKind) -> UiNodeId {
        let parent_index = parent.0 as usize;
        assert!(
            parent_index < self.nodes.len(),
            "Invalid parent id: {:?}",
            parent
        );

        let id = UiNodeId(self.nodes.len() as u32);
        let node = UiNode::new(id, Some(parent), kind);

        self.nodes.push(node);
        self.nodes[parent_index].children.push(id);
        self.dirty_layout = true;

        id
    }

    pub fn node(&self, id: UiNodeId) -> Option<&UiNode> {
        self.nodes.get(id.0 as usize)
    }

    pub fn node_mut(&mut self, id: UiNodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(id.0 as usize)
    }

    // ── Layout ───────────────────────────────────────────────────

    pub fn layout(&mut self, viewport_size: Vec2) {
        // Step 1: collect parent-child relationships and compute sizes (immutable read)
        let node_count = self.nodes.len();
        let mut sizes: Vec<(f32, f32)> = vec![(viewport_size.x, viewport_size.y); node_count];
        let mut parent_ids: Vec<Option<usize>> = vec![None; node_count];
        let mut anchors: Vec<Anchor> = vec![Anchor::TopLeft; node_count];
        let mut positions: Vec<Vec2> = vec![Vec2::ZERO; node_count];
        let mut kinds: Vec<UiNodeKind> = Vec::new();

        let viewport_center = Vec2::new(viewport_size.x * 0.5, viewport_size.y * 0.5);

        for (i, node) in self.nodes.iter().enumerate() {
            if i == 0 {
                sizes[i] = (viewport_size.x, viewport_size.y);
                parent_ids[i] = None;
                anchors[i] = Anchor::TopLeft;
                positions[i] = viewport_center;
                kinds.push(UiNodeKind::Container(ContainerKind::Free));
                continue;
            }

            let mut w = node.computed.rect.w;
            let mut h = node.computed.rect.h;

            match &node.kind {
                UiNodeKind::Image(props) => {
                    h = viewport_size.y * 0.2;
                    if let Some(handle) = &props.texture {
                        let image_arc: Arc<runa_asset::TextureAsset> = handle.clone().into();
                        if image_arc.height > 0 {
                            let aspect = image_arc.width as f32 / image_arc.height as f32;
                            w = (h * aspect).max(1.0);
                        } else {
                            w = h;
                        }
                    } else {
                        w = h;
                    }
                }
                UiNodeKind::Text(props) => {
                    h = props.font_size as f32;
                    let char_est = props.font_size as f32 * 0.5;
                    w = (props.text.len() as f32) * char_est;
                }
                UiNodeKind::Container(_) => {
                    if w <= 0.0 || h <= 0.0 {
                        w = viewport_size.x;
                        h = viewport_size.y;
                    }
                }
            }

            let min = node.layout.min_size;
            let max = node.layout.max_size;
            w = w.clamp(min.x, max.x.min(viewport_size.x));
            h = h.clamp(min.y, max.y.min(viewport_size.y));

            sizes[i] = (w, h);
            parent_ids[i] = node.parent.map(|pid| pid.0 as usize).filter(|&p| p != 0);
            anchors[i] = node.layout.anchor;
            positions[i] = node.layout.position;
            kinds.push(match &node.kind {
                UiNodeKind::Container(ck) => UiNodeKind::Container(*ck),
                UiNodeKind::Text(_) => UiNodeKind::Text(TextProps {
                    text: String::new(),
                    font: None,
                    font_size: 0,
                    color: [0.0; 4],
                    line_height: None,
                    align: crate::components::ui::TextAlign::Left,
                }),
                UiNodeKind::Image(_) => UiNodeKind::Image(ImageProps {
                    texture: None,
                    tint: [0.0; 4],
                    uv: [0.0; 4],
                }),
            });
        }

        // Step 2: build parent rects from computed sizes
        let mut rects: Vec<(f32, f32, f32, f32)> = vec![(0.0, 0.0, 0.0, 0.0); node_count];
        rects[0] = (viewport_center.x, viewport_center.y, sizes[0].0, sizes[0].1);

        for i in 1..node_count {
            let (w, h) = sizes[i];
            let anchor = anchors[i];
            let pos = positions[i];

            let parent_idx = parent_ids[i].unwrap_or(0);
            let (pcx, pcy, pw, ph) = rects[parent_idx];

            let cx = match anchor {
                Anchor::TopLeft => pcx - pw * 0.5 + pos.x + w * 0.5,
                Anchor::TopCenter => pcx + pos.x,
                Anchor::TopRight => pcx + pw * 0.5 - pos.x - w * 0.5,
                Anchor::Left => pcx - pw * 0.5 + pos.x + w * 0.5,
                Anchor::Center => pcx + pos.x,
                Anchor::Right => pcx + pw * 0.5 - pos.x - w * 0.5,
                Anchor::BottomLeft => pcx - pw * 0.5 + pos.x + w * 0.5,
                Anchor::BottomCenter => pcx + pos.x,
                Anchor::BottomRight => pcx + pw * 0.5 - pos.x - w * 0.5,
                Anchor::Stretch => pcx,
            };

            let cy = match anchor {
                Anchor::TopLeft => pcy - ph * 0.5 + pos.y + h * 0.5,
                Anchor::TopCenter => pcy - ph * 0.5 + pos.y + h * 0.5,
                Anchor::TopRight => pcy - ph * 0.5 + pos.y + h * 0.5,
                Anchor::Left => pcy + pos.y,
                Anchor::Center => pcy + pos.y,
                Anchor::Right => pcy + pos.y,
                Anchor::BottomLeft => pcy + ph * 0.5 - pos.y - h * 0.5,
                Anchor::BottomCenter => pcy + ph * 0.5 - pos.y - h * 0.5,
                Anchor::BottomRight => pcy + ph * 0.5 - pos.y - h * 0.5,
                Anchor::Stretch => pcy,
            };

            let final_w = if matches!(anchor, Anchor::Stretch) { pw - pos.x * 2.0 } else { w };
            let final_h = if matches!(anchor, Anchor::Stretch) { ph - pos.y * 2.0 } else { h };
            rects[i] = (cx, cy, final_w, final_h);
        }

        // Step 3: handle auto-layout for hbox/vbox
        for i in 0..node_count {
            if let UiNodeKind::Container(ck) = &kinds[i] {
                match ck {
                    ContainerKind::HorizontalBox | ContainerKind::VerticalBox => {
                        let children: Vec<usize> = self.nodes[i].children.iter().map(|id| id.0 as usize).collect();
                        if children.is_empty() {
                            continue;
                        }
                        let is_horizontal = matches!(ck, ContainerKind::HorizontalBox);
                        let gap = self.nodes[i].layout.gap;
                        let padding = self.nodes[i].layout.padding;
                        let (pcx, pcy, pw, ph) = rects[i];

                        let available = if is_horizontal {
                            pw - padding.left - padding.right
                        } else {
                            ph - padding.top - padding.bottom
                        };

                        let mut total = 0.0f32;
                        for &ci in &children {
                            total += if is_horizontal { rects[ci].2 } else { rects[ci].3 };
                        }

                        let spacing = if children.len() > 1 { gap * (children.len() - 1) as f32 } else { 0.0 };
                        let remaining = (available - spacing - total).max(0.0);
                        let extra = if !children.is_empty() { remaining / children.len() as f32 } else { 0.0 };

                        let mut offset = if is_horizontal {
                            pcx - pw * 0.5 + padding.left
                        } else {
                            pcy - ph * 0.5 + padding.top
                        };

                        for &ci in &children {
                            let final_size = if is_horizontal { rects[ci].2 + extra } else { rects[ci].3 + extra };
                            let half = final_size * 0.5;

                            if is_horizontal {
                                rects[ci] = (pcx - pw * 0.5 + offset + half, pcy, final_size, rects[ci].3);
                                offset += final_size + gap;
                            } else {
                                rects[ci] = (pcx, pcy - ph * 0.5 + offset + half, rects[ci].2, final_size);
                                offset += final_size + gap;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Step 4: write back to nodes
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let (cx, cy, w, h) = rects[i];
            node.computed.rect.x = cx;
            node.computed.rect.y = cy;
            node.computed.rect.w = w;
            node.computed.rect.h = h;
        }

        self.dirty_layout = false;
    }

    pub fn build_render_commands(&self, render_queue: &mut RenderQueue) {
        for node in &self.nodes {
            if !node.visible {
                continue;
            }

            if let Some(background) = node.style.background {
                let rect = node.computed.rect;
                render_queue.draw_ui_rect(
                    UiRect {
                        x: rect.x,
                        y: rect.y,
                        w: rect.w,
                        h: rect.h,
                    },
                    background,
                    node.style.z_index,
                );
            }
            match &node.kind {
                UiNodeKind::Container(_) => {}
                UiNodeKind::Image(props) => {
                    if let Some(texture) = &props.texture {
                        let rect = node.computed.rect;
                        render_queue.draw_ui_image(
                            Arc::from(texture.clone()),
                            UiRect {
                                x: rect.x,
                                y: rect.y,
                                w: rect.w,
                                h: rect.h,
                            },
                            props.tint,
                            props.uv,
                            node.style.z_index,
                        );
                    }
                }
                UiNodeKind::Text(props) => {
                    let rect = node.computed.rect;
                    render_queue.draw_ui_text(
                        props.text.clone(),
                        UiRect {
                            x: rect.x,
                            y: rect.y,
                            w: rect.w,
                            h: rect.h,
                        },
                        props.color,
                        props.font_size,
                        node.style.z_index,
                    );
                }
            }
        }
    }
}

pub enum CanvasSpace {
    Screen,
    Camera,
    World,
}

// ── Builder ──────────────────────────────────────────────────────

pub struct UiNodeBuilder<'a> {
    renderer: &'a mut UiRenderer,
    id: UiNodeId,
}

impl<'a> UiNodeBuilder<'a> {
    fn new(renderer: &'a mut UiRenderer, id: UiNodeId) -> Self {
        Self { renderer, id }
    }

    #[allow(dead_code)]
    fn new_with_click(
        renderer: &'a mut UiRenderer,
        id: UiNodeId,
        _on_click: Option<Box<dyn FnMut() + Send>>,
    ) -> Self {
        Self { renderer, id }
    }

    pub fn id(&self) -> UiNodeId {
        self.id
    }

    pub fn named(self, name: impl Into<String>) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.name = name.into();
        }
        self
    }

    pub fn with_layout(self, layout: LayoutProps) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout = layout;
        }
        self
    }

    pub fn with_style(self, style: StyleProps) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.style = style;
        }
        self
    }

    pub fn with_anchor(self, anchor: Anchor) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.anchor = anchor;
        }
        self
    }

    pub fn with_pos(self, x: f32, y: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.position = Vec2::new(x, y);
        }
        self
    }

    pub fn with_size(self, w: f32, h: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.min_size = Vec2::new(w, h);
            node.layout.max_size = Vec2::new(w, h);
        }
        self
    }

    pub fn with_min_size(self, w: f32, h: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.min_size = Vec2::new(w, h);
        }
        self
    }

    pub fn with_max_size(self, w: f32, h: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.max_size = Vec2::new(w, h);
        }
        self
    }

    pub fn with_background(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.style.background = Some([r, g, b, a]);
        }
        self
    }

    pub fn with_z_index(self, z: i16) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.style.z_index = z;
        }
        self
    }

    pub fn with_opacity(self, opacity: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.style.opacity = opacity;
        }
        self
    }

    pub fn with_gap(self, gap: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.gap = gap;
        }
        self
    }

    pub fn with_padding(self, l: f32, t: f32, r: f32, b: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.padding = crate::components::ui::EdgeInsets { left: l, top: t, right: r, bottom: b };
        }
        self
    }

    pub fn with_margin(self, l: f32, t: f32, r: f32, b: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.layout.margin = crate::components::ui::EdgeInsets { left: l, top: t, right: r, bottom: b };
        }
        self
    }

    /// For text nodes: set font size
    pub fn with_font_size(self, size: u16) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            if let UiNodeKind::Text(ref mut props) = node.kind {
                props.font_size = size;
            }
        }
        self
    }

    /// For text nodes: set color
    pub fn with_text_color(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            if let UiNodeKind::Text(ref mut props) = node.kind {
                props.color = [r, g, b, a];
            }
        }
        self
    }

    /// For image nodes: set tint
    pub fn with_tint(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            if let UiNodeKind::Image(ref mut props) = node.kind {
                props.tint = [r, g, b, a];
            }
        }
        self
    }

    /// For image nodes: set texture handle
    pub fn with_texture(self, texture: runa_asset::Handle<runa_asset::TextureAsset>) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            if let UiNodeKind::Image(ref mut props) = node.kind {
                props.texture = Some(texture);
            }
        }
        self
    }

    /// Set visibility
    pub fn visible(self, visible: bool) -> Self {
        if let Some(node) = self.renderer.node_mut(self.id) {
            node.visible = visible;
        }
        self
    }

    /// Returns the node's computed rect (must call layout() first)
    pub fn rect(&self) -> Option<crate::components::ui::UiRect> {
        self.renderer.node(self.id).map(|n| n.computed.rect)
    }
}
