pub mod input;
pub mod layout;
pub mod node;
pub mod style;
pub mod builder;
pub mod component;

use std::{collections::HashSet, io::Stdout, ops::{Deref, DerefMut}, time::Duration};

use bevy::{app::ScheduleRunnerPlugin, ecs::{query::QueryData}, prelude::*};
use ratatui::{TerminalOptions, crossterm::{self, ExecutableCommand, event::{Event, KeyCode}, terminal::{EnterAlternateScreen, LeaveAlternateScreen}}, prelude::CrosstermBackend};

use crate::{component::TerminalComponentPlugin, input::TerminalInput, layout::{GlobalRect, Layout, Padding, LayoutState, TerminalLayoutPlugin, TerminalSize}, node::{Node, NodePlugin, NullComponent, query::NodeEntityMut}, style::Style};

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
        let size = terminal.size().unwrap();
        
        if matches!(self.viewport, ratatui::Viewport::Fullscreen) {
            std::io::stdout().execute(EnterAlternateScreen).expect("Unable to enter alternate screen.");
        }
        
        terminal.hide_cursor().unwrap();
        
        app
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(1.0 / 120.0)))
            .add_plugins(NodePlugin)
            .add_plugins(TerminalLayoutPlugin)
            .add_plugins(AssetPlugin::default())
            .add_plugins(TerminalComponentPlugin)
            .add_plugins(TaskPoolPlugin::default())
            
            .insert_resource(terminal)
            .insert_resource(viewport)
            .insert_resource(TerminalSize(size.width, size.height))
            
            .add_message::<TerminalMessage>()
        
            .add_systems(First, poll_app)
            .add_systems(PostUpdate, close_on_esc)
            .add_systems(Last, (flush, cleanup))
            .add_systems(Update, debug)
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn poll_app(
    mut message_writer : MessageWriter<TerminalMessage>,
    mut nodes : Query<NodeEntityMut>
) -> Result<(), BevyError> {
    if crossterm::event::poll(Duration::from_secs(0)).unwrap_or(false) {
        let event = crossterm::event::read()?;
        if matches!(event, Event::Resize(_, _)) { nodes.iter_mut().for_each(|mut node| *node.rect_state = LayoutState::Dirty);}
        message_writer.write(TerminalMessage(event));
    }
    
    Ok(())
}

pub fn flush(
    mut terminal : ResMut<Terminal>
) {
    terminal.autoresize().unwrap();
    
    terminal.flush().unwrap();
    
    terminal.swap_buffers();
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
            std::io::stdout().execute(LeaveAlternateScreen).expect("Unable to leave alternate screen.");
        }
        ratatui::restore();
    }
}

pub fn debug(
    mut nodes : Query<NodeEntityMut>
) {
    for node in nodes.iter_mut().filter(|node| node.has_id("main")) {
        let Some(mut style) = node.style else {continue};
        style.set_fg(ratatui::style::Color::Green);
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
//        Should Restore
//==============================================================================================

#[derive(Resource)]
pub struct Viewport(pub ratatui::Viewport);