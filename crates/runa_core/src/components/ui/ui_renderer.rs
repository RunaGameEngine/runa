use std::sync::Arc;

use glam::Vec2;
use runa_render_api::{command::UiRect, RenderQueue};

use crate::components::ui::{
    Anchor, ContainerKind, ImageProps, TextProps, UiNode, UiNodeId, UiNodeKind,
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
        let root_node = UiNode::new(root, None, UiNodeKind::Container(ContainerKind::Free));

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
        let root_node = UiNode::new(self.root, None, UiNodeKind::Container(ContainerKind::Free));
        self.nodes.clear();
        self.nodes.push(root_node);
        self.dirty_layout = true;
    }

    pub fn root(&self) -> UiNodeId {
        self.root
    }

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

    pub fn layout(&mut self, viewport_size: Vec2) {
        use runa_asset::TextureAsset;
        // Simple layout implementation:
        // - Images: size auto-derived from texture dimensions (20% of viewport height)
        // - Text: width estimated from chars * font_size * 0.5, height = font_size
        // - Position: computed based on anchor and layout.position (position is offset in pixels)

        for (i, node) in self.nodes.iter_mut().enumerate() {
            if i == 0 {
                // root: occupy full viewport
                node.computed.rect.w = viewport_size.x;
                node.computed.rect.h = viewport_size.y;
                node.computed.rect.x = viewport_size.x * 0.5;
                node.computed.rect.y = viewport_size.y * 0.5;
                continue;
            }

            let mut w = node.computed.rect.w;
            let mut h = node.computed.rect.h;

            match &node.kind {
                UiNodeKind::Image(props) => {
                    // Default sizing: 20% of viewport height
                    h = viewport_size.y * 0.2;
                    if let Some(handle) = &props.texture {
                        // Access texture dimensions via Handle -> Arc<TextureAsset>
                        let image_arc: Arc<TextureAsset> = handle.clone().into();
                        if image_arc.height > 0 {
                            let aspect = image_arc.width as f32 / image_arc.height as f32;
                            w = (h * aspect).max(1.0);
                        } else {
                            w = h; // fallback square
                        }
                    } else {
                        w = h; // fallback square
                    }
                }
                UiNodeKind::Text(props) => {
                    h = props.font_size as f32;
                    let char_est = props.font_size as f32 * 0.5;
                    w = (props.text.len() as f32) * char_est;
                }
                UiNodeKind::Container(_) => {
                    // For free containers we don't change size unless constraints exist
                    if w == 0.0 || h == 0.0 {
                        w = viewport_size.x;
                        h = viewport_size.y;
                    }
                }
            }

            // Apply min/max constraints from layout props
            let min = node.layout.min_size;
            let max = node.layout.max_size;
            w = w.clamp(min.x, max.x.min(viewport_size.x));
            h = h.clamp(min.y, max.y.min(viewport_size.y));

            // Compute center based on anchor
            let pos = node.layout.position;
            let cx = match node.layout.anchor {
                Anchor::TopLeft => pos.x + w * 0.5,
                Anchor::TopCenter => viewport_size.x * 0.5 + pos.x,
                Anchor::TopRight => viewport_size.x - (pos.x + w * 0.5),
                Anchor::Left => pos.x + w * 0.5,
                Anchor::Center => viewport_size.x * 0.5 + pos.x,
                Anchor::Right => viewport_size.x - (pos.x + w * 0.5),
                Anchor::BottomLeft => pos.x + w * 0.5,
                Anchor::BottomCenter => viewport_size.x * 0.5 + pos.x,
                Anchor::BottomRight => viewport_size.x - (pos.x + w * 0.5),
                Anchor::Stretch => viewport_size.x * 0.5,
            };

            let cy = match node.layout.anchor {
                Anchor::TopLeft => pos.y + h * 0.5,
                Anchor::TopCenter => pos.y + h * 0.5,
                Anchor::TopRight => pos.y + h * 0.5,
                Anchor::Left => viewport_size.y * 0.5 + pos.y,
                Anchor::Center => viewport_size.y * 0.5 + pos.y,
                Anchor::Right => viewport_size.y * 0.5 + pos.y,
                Anchor::BottomLeft => viewport_size.y - (pos.y + h * 0.5),
                Anchor::BottomCenter => viewport_size.y - (pos.y + h * 0.5),
                Anchor::BottomRight => viewport_size.y - (pos.y + h * 0.5),
                Anchor::Stretch => viewport_size.y * 0.5,
            };

            node.computed.rect.w = w;
            node.computed.rect.h = h;
            node.computed.rect.x = cx;
            node.computed.rect.y = cy;
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
