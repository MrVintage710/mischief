use std::{collections::{HashMap, HashSet}, marker::PhantomData};

use bevy::{prelude::*};
use ratatui::{widgets::Widget};
use taffy::{AvailableSpace, LengthPercentage, LengthPercentageAuto, NodeId, TaffyTree};

use crate::{TerminalMessage, node::{self, query::{NodeEntityMut, NodeFindFamilyAbility, NodeFindLeavesAbility, NodeFindRootsAbility, NodeFindSiblingAbility}}};

//==============================================================================================
//        TerminalRenderPlugin
//==============================================================================================

pub struct TerminalLayoutPlugin;

impl Plugin for TerminalLayoutPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Last,
                (
                    resize,
                    calc_layout.run_if(|states : Query<&LayoutState>| states.iter().any(|s| s.is_dirty()))
                ).chain()
            )
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

// pub fn calc_rects(
//     mut nodes : Query<NodeEntityMut>,
//     terminal_size: Res<TerminalSize>
// ) {
//     fn calc_layout(entity : Entity, nodes : &mut Query<NodeEntityMut>, terminal_size : &TerminalSize) -> (GlobalRect, Layout, Padding) {
        
//         // Get the global bounds just in case
//         let global_bounds = {
//             GlobalRect(ratatui::layout::Rect { x: 0, y: 0, width: terminal_size.0, height: terminal_size.1 })
//         };
        
//         let Some(current) = nodes.get(entity).ok() else { return (global_bounds, Layout::Relative, Padding::default()) };
        
//         if current.rect_state.is_ok() { return (current.global_rect.clone(), *current.layout, current.padding.cloned().unwrap_or_default()) }
        
//         let parent = current.parent.cloned().unwrap_or(ChildOf(Entity::PLACEHOLDER));
//         let (parent_bounds, parent_layout, parent_padding) = calc_layout(parent.0, nodes, terminal_size);
        
//         let bounds = ratatui::layout::Rect {
//             x: parent_bounds.0.x + parent_padding.left,
//             y: parent_bounds.0.y + parent_padding.top,
//             width: parent_bounds.0.width.saturating_sub(parent_padding.right.saturating_add(parent_padding.left)),
//             height: parent_bounds.0.height.saturating_sub(parent_padding.bottom.saturating_add(parent_padding.top)),
//         };
        
//         let siblings = nodes.find_siblings(entity);
//         let sibling_index = nodes.sibling_index(entity);
//         let mut child = nodes.get_mut(entity).unwrap();
        
//         match parent_layout {
//             Layout::Relative => {
                
//                 child.global_rect.0 = ratatui::layout::Rect {
//                     x: bounds.x + child.rect.x.get_value(bounds.width), 
//                     y: bounds.y + child.rect.y.get_value(bounds.height), 
//                     width: child.rect.width.get_value(bounds.width), 
//                     height: child.rect.height.get_value(bounds.height)
//                 };
                
//                 *child.layout_state = LayoutState::Ok;
//             },
//             Layout::Flex(flex_options) => {
//                 let FlexOptions { gap, direction } = flex_options;
//                 match direction {
//                     Direction::Vertical => {
                        
//                     },
//                     Direction::Horizontal => todo!(),
//                 }
//             },
//         }
        
//         (*child.global_rect, *child.layout, child.padding.as_deref().cloned().unwrap_or_default())
//     }
    
//     let entities = nodes.iter().map(|node| node.entity).collect::<Vec<_>>();
    
//     for e in entities {
//         calc_layout(e, &mut nodes, terminal_size.as_ref());
//     }
// }

pub fn calc_layout(
    mut nodes : Query<NodeEntityMut>,
    terminal_size: Res<TerminalSize>
) {

    pub fn fill_tree_r(current : Entity, nodes : &mut Query<NodeEntityMut>, tree : &mut TaffyTree<Entity>, lookup : &mut HashMap<Entity, NodeId>) -> Option<NodeId> {
        if let Some(node_id) = lookup.get(&current) {return Some(*node_id)}
        let node = nodes.get(current).ok()?;

        //Get taffy Styleing for style
        let style : taffy::Style = (*node.style).into();

        let children = node.children_cloned();
        let children_nodes = children.into_iter().filter_map(|child| fill_tree_r(child, nodes, tree, lookup)).collect::<Vec<_>>();

        let node_id = if children_nodes.is_empty() {
            tree.new_leaf(style).ok()?
        } else {
            tree.new_with_children(style, children_nodes.as_slice()).ok()?
        };
        
        lookup.insert(current, node_id);

        return Some(node_id)
    }
    
    let mut node_lookup = HashMap::<Entity, NodeId>::new();
    let mut tree : TaffyTree<Entity> = TaffyTree::new();
    tree.enable_rounding();

    let node_entities = nodes.iter().map(|n| n.entity).collect::<Vec<_>>();
    for node in node_entities.iter() {
        fill_tree_r(*node, &mut nodes, &mut tree, &mut node_lookup);
    }

    let dirty_nodes = nodes.iter().filter(|n| n.is_layout_dirty()).map(|n| n.entity).collect::<Vec<_>>();
    let mut nodes_needing_update = HashSet::new();
    
    for root in nodes.find_roots().iter() {
        let Ok(mut node) = nodes.get_mut(*root) else { continue };
        let Some(node_id) = node_lookup.get(&node.entity) else { continue };
        tree.compute_layout(*node_id, taffy::Size { 
            width: AvailableSpace::Definite(terminal_size.0 as f32), 
            height: AvailableSpace::Definite(terminal_size.1 as f32)
        }).unwrap();
        node.set_layout_ok();
        for child in nodes.find_family(*root) {
            nodes_needing_update.insert(child);
        }
    }
    
    for node_entity in nodes_needing_update {
        let Ok(mut node) = nodes.get_mut(node_entity) else { continue };
        let Some(node_id) = node_lookup.get(&node.entity) else { continue };
        let Ok(layout) = tree.layout(*node_id) else { continue };
        let rect = ratatui::layout::Rect { 
            x: layout.location.x as u16, 
            y: layout.location.y as u16, 
            width: layout.size.width as u16, 
            height: layout.size.height as u16 
        };
        node.rect.0 = rect;
    }
}

pub fn resize(
    mut terminal_size : ResMut<TerminalSize>,
    mut event : MessageReader<TerminalMessage>
) {
    for event in event.read() {
        let ratatui::crossterm::event::Event::Resize(width, height) = event.0 else { continue };
        *terminal_size = TerminalSize(width, height);
    }
}

//==============================================================================================
//        WidgetRenderer
//==============================================================================================

#[derive(Component)]
pub struct WidgetRenderer<W : Widget>(PhantomData<W>);

impl <W : Widget> Default for WidgetRenderer<W> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl <W : Widget> From<W> for WidgetRenderer<W> {
    fn from(_: W) -> Self {
        WidgetRenderer::<W>::default()
    }
}

//==============================================================================================
//        Rect
//==============================================================================================

#[derive(Component, Default, Clone, Copy, Debug)]
#[require(LayoutState)]
pub struct Rect(pub ratatui::layout::Rect);

//==============================================================================================
//        NodeState
//==============================================================================================

#[derive(Component, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayoutState {
    #[default]
    Dirty,
    Ok
}

impl LayoutState {
    /// Returns `true` if the rect state is [`Dirty`].
    ///
    /// [`Dirty`]: RectState::Dirty
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        matches!(self, Self::Dirty)
    }

    /// Returns `true` if the rect state is [`Ok`].
    ///
    /// [`Ok`]: RectState::Ok
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    pub fn set_dirty(&mut self) {
        std::mem::replace(self, LayoutState::Dirty);
    }

    pub fn set_ok(&mut self) {
        std::mem::replace(self, LayoutState::Ok);
    }
}

//==============================================================================================
//        TerminalSize
//==============================================================================================

#[derive(Debug, Resource)]
pub struct TerminalSize(pub u16, pub u16);