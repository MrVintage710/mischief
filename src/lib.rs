pub mod input;
pub mod layout;
pub mod node;
pub mod style;
pub mod builder;
pub mod component;

use std::{collections::HashSet, io::Stdout, ops::{Deref, DerefMut}, time::Duration};

use bevy::{app::ScheduleRunnerPlugin, ecs::{query::QueryData, resource}, prelude::*};
use ratatui::{TerminalOptions, crossterm::{self, ExecutableCommand, event::{Event, KeyCode}, terminal::{EnterAlternateScreen, LeaveAlternateScreen}}, prelude::CrosstermBackend};

use crate::{component::TerminalComponentPlugin, input::TerminalInput, layout::{GlobalRect, Layout, Padding, RectState, TerminalLayoutPlugin}, node::{Node, NodePlugin, NullComponent}, style::Style};

//==============================================================================================
//        TerminalAppPlugin
//==============================================================================================

pub struct MischiefPlugin {
    viewport : ratatui::Viewport,
}

impl MischiefPlugin {
    pub fn new() -> Self {
        Self { viewport : ratatui::Viewport::Fullscreen }
    }
    
    pub fn inline(mut self, height : u16) -> Self {
        self.viewport = ratatui::Viewport::Inline(height);
        self
    }
}

impl Plugin for MischiefPlugin{
    fn build(&self, app: &mut App) {
        let mut terminal = Terminal(ratatui::init_with_options(TerminalOptions { viewport : self.viewport.clone() }));
        let viewport = Viewport(self.viewport.clone());
        
        if matches!(self.viewport, ratatui::Viewport::Fullscreen) {
            // std::io::stdout().execute(EnterAlternateScreen).expect("Unable to enter alternate screen.");
        }
        
        terminal.hide_cursor().unwrap();
        
        app
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(1.0 / 60.0)))
            .add_plugins(NodePlugin)
            .add_plugins(TerminalLayoutPlugin)
            .add_plugins(AssetPlugin::default())
            .add_plugins(TerminalComponentPlugin)
            .add_plugins(TaskPoolPlugin::default())
            
            .insert_resource(terminal)
            .insert_resource(viewport)
            
            .add_message::<TerminalMessage>()
        
            .add_systems(First, poll_app)
            .add_systems(PostUpdate, close_on_esc)
            .add_systems(Last, (flush, cleanup))
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn poll_app(
    mut message_writer : MessageWriter<TerminalMessage>
) -> Result<(), BevyError> {
    if crossterm::event::poll(Duration::from_secs(0)).unwrap_or(false) {
        message_writer.write(TerminalMessage(crossterm::event::read()?));
    }
    
    Ok(())
}

pub fn flush(
    mut terminal : ResMut<Terminal>
) {
    terminal.flush().unwrap();
}

pub fn close_on_esc(
    mut input : TerminalInput,
    mut exit_message : MessageWriter<AppExit>
) {
    if input.pressed(KeyCode::Esc) {
        exit_message.write(AppExit::Success);
    }
}

pub fn cleanup(
    exit_events: MessageReader<AppExit>,
    viewport : Res<Viewport>
) {
    if !exit_events.is_empty() {
        if matches!(viewport.0, ratatui::Viewport::Fullscreen) {
            // std::io::stdout().execute(LeaveAlternateScreen).expect("Unable to leave alternate screen.");
        }
        ratatui::restore();
    }
}

//==============================================================================================
//        TerminalMessage
//==============================================================================================

#[derive(Message)]
pub struct TerminalMessage(Event);

impl DerefMut for TerminalMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for TerminalMessage {
    type Target = Event;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//==============================================================================================
//        Terminal
//==============================================================================================

#[derive(Resource)]
pub struct Terminal(ratatui::Terminal<CrosstermBackend<Stdout>>);

impl DerefMut for Terminal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Terminal {
    type Target = ratatui::Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//==============================================================================================
//        NodeQuery
//==============================================================================================

#[derive(QueryData)]
pub struct GeneralNodeQuery {
    pub entity : Entity,
    pub global_rect: &'static GlobalRect,
    pub rect : &'static crate::layout::Rect,
    pub rect_state : &'static RectState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub layout : &'static Layout,
    pub padding : Option<&'static Padding>,
    pub style : Option<&'static Style>,
}

#[derive(QueryData)]
pub struct NodeQuery<N : Node> {
    pub entity : Entity,
    pub global_rect: &'static GlobalRect,
    pub rect : &'static crate::layout::Rect,
    pub rect_state : &'static RectState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub layout : &'static Layout,
    pub padding : Option<&'static Padding>,
    pub style : Option<&'static Style>,
    pub component : &'static N
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct NodeQueryMut {
    pub entity : Entity,
    pub global_rect: &'static mut GlobalRect,
    pub rect : &'static crate::layout::Rect,
    pub rect_state : &'static mut RectState,
    pub parent : Option<&'static ChildOf>,
    pub layout : &'static Layout,
    pub padding : Option<&'static Padding>
}

pub fn find_roots(query : &Query<GeneralNodeQuery>) -> Vec<Entity> {
    
    //Define recursive function
    fn find_roots_r(current : Entity, query : &Query<GeneralNodeQuery>, visited : &mut HashSet<Entity>, roots : &mut Vec<Entity>) {
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
    
    for node in query.iter() {
        find_roots_r(node.entity, query, &mut visited, &mut roots);
    }
    
    roots
}

//==============================================================================================
//        Should Restore
//==============================================================================================

#[derive(Resource)]
pub struct Viewport(pub ratatui::Viewport);