use bevy::{ecs::query::QueryData, platform::collections::{Equivalent, HashSet}, prelude::*};

use crate::{layout::{GlobalRect, Layout, LayoutState, Padding}, node::Node, style::Style};

//==============================================================================================
//        NodeEntity
//==============================================================================================

#[derive(QueryData)]
pub struct NodeEntity {
    pub entity : Entity,
    pub global_rect: &'static GlobalRect,
    pub rect : &'static crate::layout::Rect,
    pub rect_state : &'static LayoutState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub layout : &'static Layout,
    pub padding : Option<&'static Padding>,
    pub style : Option<&'static Style>,
    pub id : Option<&'static Name>
}

impl <'w, 's> NodeEntityItem<'w, 's> {
    pub fn has_id(&self, id : &str) -> bool {
        let Some(name) = self.id else {return false };
        name.as_str() == id
    }
    
    pub fn parent(&self) -> Option<Entity> {
        self.parent.map(|p| p.0)
    }
    
    pub fn children(&self) -> Option<&Children> {
        self.children
    }

    pub fn children_cloned(&self) -> Vec<Entity> {
        let Some(children) = self.children else { return Vec::new() };
        children.iter().map(|e| e.clone()).collect()
    }
}


//==============================================================================================
//        NodeEntityMut
//==============================================================================================

#[derive(QueryData)]
#[query_data(mutable)]
pub struct NodeEntityMut {
    pub entity : Entity,
    pub global_rect: &'static mut GlobalRect,
    pub rect : &'static mut crate::layout::Rect,
    pub rect_state : &'static mut LayoutState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub layout : &'static mut Layout,
    pub padding : Option<&'static mut Padding>,
    pub style : Option<&'static mut Style>,
    pub id : Option<&'static Name>
}

impl <'w, 's> NodeEntityMutReadOnlyItem<'w, 's> {
    pub fn has_id(&self, id : &str) -> bool {
        let Some(name) = self.id else {return false };
        name.as_str() == id
    }
    
    pub fn parent(&self) -> Option<Entity> {
        self.parent.map(|p| p.0)
    }
    
    pub fn children(&self) -> Option<&Children> {
        self.children
    }

    pub fn children_cloned(&self) -> Vec<Entity> {
        let Some(children) = self.children else { return Vec::new() };
        children.iter().map(|e| e.clone()).collect()
    }
}

impl <'w, 's> NodeEntityMutItem<'w, 's> {
    pub fn has_id(&self, id : &str) -> bool {
        let Some(name) = self.id else {return false };
        name.as_str() == id
    }
    
    pub fn parent(&self) -> Option<Entity> {
        self.parent.map(|p| p.0)
    }
    
    pub fn children(&self) -> Option<&Children> {
        self.children
    }

    pub fn children_cloned(&self) -> Vec<Entity> {
        let Some(children) = self.children else { return Vec::new() };
        children.iter().map(|e| e.clone()).collect()
    }
}

//==============================================================================================
//        NodeFindRootsAbility
//==============================================================================================

pub trait NodeFindRootsAbility : Sized {
    fn find_roots(&self) -> Vec<Entity>;
}

impl <'w, 's> NodeFindRootsAbility for Query<'w, 's, NodeEntity> {
    fn find_roots(&self) -> Vec<Entity>  {
        fn find_roots_r(current : Entity, query : &Query<'_, '_, NodeEntity>, visited : &mut HashSet<Entity>, roots : &mut Vec<Entity>) {
            if visited.contains(&current) { return }
            let Ok(current_item) = query.get(current) else { return };
            let Some(parent) = current_item.parent else {
                roots.push(current);
                return;
            };
            let Ok(parent) = query.get(parent.0) else {
                roots.push(current);
                return;
            };
            find_roots_r(parent.entity, query, visited, roots);
        }
        
        let mut roots = Vec::new();
        let mut visited = HashSet::<Entity>::new();
        
        for node in self.iter() {
            find_roots_r(node.entity, self, &mut visited, &mut roots);
        }
        
        roots
    }
}

impl <'w, 's> NodeFindRootsAbility for Query<'w, 's, NodeEntityMut> {
    fn find_roots(&self) -> Vec<Entity>  {
        fn find_roots_r(current : Entity, query : &Query<'_, '_, NodeEntityMut>, visited : &mut HashSet<Entity>, roots : &mut Vec<Entity>) {
            if visited.contains(&current) { return }
            let Ok(current_item) = query.get(current) else { return };
            let Some(parent) = current_item.parent else {
                roots.push(current);
                return;
            };
            let Ok(parent) = query.get(parent.0) else {
                roots.push(current);
                return;
            };
            find_roots_r(parent.entity, query, visited, roots);
        }
        
        let mut roots = Vec::new();
        let mut visited = HashSet::<Entity>::new();
        
        for node in self.iter() {
            find_roots_r(node.entity, self, &mut visited, &mut roots);
        }
        
        roots
    }
}

//==============================================================================================
//        NodeFindSiblingsAbility
//==============================================================================================

pub trait NodeFindSiblingAbility : Sized {
    fn find_siblings(&self, entity : Entity) -> Vec<Entity>;
    
    fn sibling_index(&self, entity : Entity) -> usize;
}

impl <'w, 's> NodeFindSiblingAbility for Query<'w, 's, NodeEntity> {
    fn find_siblings(&self, entity : Entity) -> Vec<Entity> {
        let Some(node) = self.get(entity).ok() else { return Vec::new() };
        let Some(parent) = node.parent() else { return Vec::new() };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return Vec::new() };
        let Some(children) = parent.children() else { return Vec::new() };
        children.iter().filter(|c| *c != entity).collect()
    }

    fn sibling_index(&self, entity : Entity) -> usize {
        let Some(node) = self.get(entity).ok() else { return 0 };
        let Some(parent) = node.parent() else { return 0 };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return 0 };
        let Some(children) = parent.children() else { return 0 };
        children.iter().position(|c| c == entity).unwrap_or_default()
    }
}

impl <'w, 's> NodeFindSiblingAbility for Query<'w, 's, NodeEntityMut> {
    fn find_siblings(&self, entity : Entity) -> Vec<Entity> {
        let Some(node) = self.get(entity).ok() else { return Vec::new() };
        let Some(parent) = node.parent() else { return Vec::new() };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return Vec::new() };
        let Some(children) = parent.children() else { return Vec::new() };
        children.iter().filter(|c| *c != entity).collect()
    }
    
    fn sibling_index(&self, entity : Entity) -> usize {
        let Some(node) = self.get(entity).ok() else { return 0 };
        let Some(parent) = node.parent() else { return 0 };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return 0 };
        let Some(children) = parent.children() else { return 0 };
        children.iter().position(|c| c == entity).unwrap_or_default()
    }
}

//==============================================================================================
//        NodeFindLeafAbility
//==============================================================================================

pub trait NodeFindLeavesAbility : Sized {
    fn find_leaves(&self) -> Vec<Entity>;
}

impl <'w, 's> NodeFindLeavesAbility for Query<'w, 's, NodeEntity> {
    fn find_leaves(&self) -> Vec<Entity> {
        self.iter().filter(|node| node.children.is_none()).map(|node| node.entity).collect()
    }
}

impl <'w, 's> NodeFindLeavesAbility for Query<'w, 's, NodeEntityMut> {
    fn find_leaves(&self) -> Vec<Entity> {
        self.iter().filter(|node| node.children.is_none()).map(|node| node.entity).collect()
    }
} 