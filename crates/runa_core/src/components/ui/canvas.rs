use std::sync::Arc;

use glam::Vec2;
use runa_render_api::{command::UiRect, RenderQueue};

use crate::components::ui::{ContainerKind, ImageProps, TextProps, UiNode, UiNodeId, UiNodeKind};

pub struct Canvas {
    pub space: CanvasSpace,
    pub nodes: Vec<UiNode>,
    pub root: UiNodeId,
    pub dirty_layout: bool,
}

impl Canvas {
    pub fn new(space: CanvasSpace) -> Self {
        let root = UiNodeId(0);
        let root_node = UiNode::new(root, None, UiNodeKind::Container(ContainerKind::Free));

        Self {
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

    pub fn layout(&mut self, _viewport_size: Vec2) {
        // TODO: implkement measure + arragne
        self.dirty_layout = false
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
}
