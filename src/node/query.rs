use bevy::{ecs::query::QueryData, platform::collections::{Equivalent, HashSet}, prelude::*};

use crate::{layout::{Rect, LayoutState}, node::Node, style::Style};

//==============================================================================================
//        NodeEntity
//==============================================================================================

#[derive(QueryData)]
pub struct NodeEntity {
    pub entity : Entity,
    pub rect: &'static Rect,
    pub layout_state : &'static LayoutState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub style : &'static Style,
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

    pub fn is_layout_dirty(&self) -> bool {
        self.layout_state.is_dirty()
    }
}


//==============================================================================================
//        NodeEntityMut
//==============================================================================================

#[derive(QueryData)]
#[query_data(mutable)]
pub struct NodeEntityMut {
    pub entity : Entity,
    pub rect: &'static mut Rect,
    pub layout_state : &'static mut LayoutState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub style : &'static mut Style,
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

    pub fn is_layout_dirty(&self) -> bool {
        self.layout_state.is_dirty()
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

    pub fn is_layout_dirty(&self) -> bool {
        self.layout_state.is_dirty()
    }

    pub fn set_layout_dirty(&mut self) {
        self.layout_state.set_dirty();
    }

    pub fn set_layout_ok(&mut self) {
        self.layout_state.set_ok();
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

//==============================================================================================
//        NodeIsDirtyAbility
//==============================================================================================

pub trait NodeCheckLayoutStateAbility : Sized {
    fn is_layout_dirty(&self, entity : Entity) -> bool;

    fn is_layout_ok(&self, entity : Entity) -> bool;
}

impl <'w, 's> NodeCheckLayoutStateAbility for Query<'w, 's, NodeEntity> {
    fn is_layout_dirty(&self, entity : Entity) -> bool {
        let Ok(node) = self.get(entity) else { return false };
        node.layout_state.is_dirty()
    }

    fn is_layout_ok(&self, entity : Entity) -> bool {
        let Ok(node) = self.get(entity) else { return false };
        node.layout_state.is_ok()
    }
}

impl <'w, 's> NodeCheckLayoutStateAbility for Query<'w, 's, NodeEntityMut> {
    fn is_layout_dirty(&self, entity : Entity) -> bool {
        let Ok(node) = self.get(entity) else { return false };
        node.layout_state.is_dirty()
    }

    fn is_layout_ok(&self, entity : Entity) -> bool {
        let Ok(node) = self.get(entity) else { return false };
        node.layout_state.is_ok()
    }
}

//==============================================================================================
//        NodeFindFamilyAbility
//==============================================================================================

pub trait NodeFindFamilyAbility : Sized {
    fn find_family(&self, entity : Entity) -> Vec<Entity>;
}

impl <'w, 's> NodeFindFamilyAbility for Query<'w, 's, NodeEntity> {
    fn find_family(&self, entity : Entity) -> Vec<Entity> {
        let Ok(node) = self.get(entity) else { return  vec![] };
        let Some(children) = node.children else {return vec![entity] };
        let mut result = vec![entity];
        for child in children.iter() {
            result.push(child);
            result.append(&mut self.find_family(child));
        }
        result
    }
}

impl <'w, 's> NodeFindFamilyAbility for Query<'w, 's, NodeEntityMut> {
    fn find_family(&self, entity : Entity) -> Vec<Entity> {
        let Ok(node) = self.get(entity) else { return  vec![] };
        let Some(children) = node.children else {return vec![entity] };
        let mut result = vec![entity];
        for child in children.iter() {
            result.push(child);
            result.append(&mut self.find_family(child));
        }
        result
    }
}