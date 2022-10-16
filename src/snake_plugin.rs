use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::{
    snake::{Direction, Position},
    snake_plugin::events::FoodAteEvent,
};

use self::{components::*, events::GameTick, resources::GameTimerResource};

use self::{events::GameOverEvent, resources::DrawConfigurationResource};

use crate::snake::{SnakeGame, SnakeGameSnapshot};

// Walls and foods never overlap,
// So we don't care if they are the same value
const WALL_Z: f32 = 0.;
const FOOD_Z: f32 = 0.;
// Snake can overlap with foods,
// so should be bigger
const SNAKE_Z: f32 = 1.;

pub struct SnakePlugin {
    pub rect: UiRect<f32>,
    pub level: &'static str,
}

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        let snake_game: SnakeGame = self.level.parse().unwrap();

        let dim = snake_game.dimension();

        let cell_width = (self.rect.right - self.rect.left) / dim.0 as f32;
        let cell_height = (self.rect.top - self.rect.bottom) / dim.1 as f32;
        let cell_size = cell_height.min(cell_width);

        let snapshot = snake_game.snapshot();

        app.insert_resource(snapshot)
            .insert_resource(snake_game)
            .insert_resource(Direction::Up)
            .insert_resource(DrawConfigurationResource {
                cell_size,
                half_cell: cell_size / 2.,
                origin: (self.rect.bottom, self.rect.left),
            })
            .insert_resource(GameTimerResource(Timer::from_seconds(0.5, true)))
            .init_resource::<PbrBundles>()
            .add_event::<GameOverEvent>()
            .add_event::<FoodAteEvent>()
            .add_event::<GameTick>()
            .add_startup_system(draw_field)
            .add_startup_system(draw_snake)
            .add_system(game_tick)
            .add_system(change_direction)
            .add_system(play.label("play"))
            .add_system(update_snake_head.after("play"))
            .add_system(update_snake_body.after("play"))
            .add_system(update_food.after("play"))
            .add_system(game_over.after("play"));
    }
}

fn game_tick(
    time: Res<Time>,
    mut game_timers: ResMut<GameTimerResource>,
    mut tick_event_writer: EventWriter<GameTick>,
) {
    let game_timers = &mut game_timers.0;
    if !game_timers.tick(time.delta()).finished() {
        return;
    }

    debug!("tick!");

    tick_event_writer.send(GameTick);
}

fn play(
    current_direction: Res<Direction>,
    mut tick_event: EventReader<GameTick>,
    mut snake_game: ResMut<SnakeGame>,
    mut snapshot: ResMut<SnakeGameSnapshot>,
    mut game_over_event_writer: EventWriter<GameOverEvent>,
) {
    if tick_event.iter().count() == 0 {
        return;
    }

    let result = snake_game.play(*current_direction);

    *snapshot = match result {
        Ok(snapshot) => snapshot,
        Err(error) => {
            game_over_event_writer.send(GameOverEvent { error });
            return;
        }
    };
}

fn update_snake_head(
    draw_configuration: Res<DrawConfigurationResource>,
    snapshot: Res<SnakeGameSnapshot>,
    mut head_snake_query: Query<&mut Transform, With<HeadSnakeComponent>>,
) {
    if !snapshot.is_changed() {
        return;
    }

    let position = &snapshot.snake[0];
    move_to(
        &mut head_snake_query.iter_mut().next().unwrap(),
        position,
        &draw_configuration,
    );
}

fn update_snake_body(
    mut commands: Commands,
    bundles: Res<PbrBundles>,
    draw_configuration: Res<DrawConfigurationResource>,
    snapshot: Res<SnakeGameSnapshot>,
    mut body_snake_query: Query<(Entity, &mut Transform), With<BodySnakeComponent>>,
) {
    if !snapshot.is_changed() {
        return;
    }

    let mut bodies = body_snake_query.iter_mut();
    for snake_body_position in snapshot.snake.iter().skip(1) {
        match bodies.next() {
            Some(mut e) => {
                move_to(&mut e.1, snake_body_position, &draw_configuration);
            }
            None => {
                spawn_snake_body(
                    &mut commands,
                    &bundles,
                    snake_body_position,
                    &draw_configuration,
                );
            }
        }
    }

    // The remain pieces are unknown: we should remove then
    // Theoretically there're not remains
    bodies.for_each(|p| {
        commands.entity(p.0).despawn();
    });
}

fn update_food(
    draw_configuration: Res<DrawConfigurationResource>,
    snapshot: Res<SnakeGameSnapshot>,
    mut food_query: Query<&mut Transform, With<FoodComponent>>,
    mut food_ate_event_writer: EventWriter<FoodAteEvent>,
) {
    if !snapshot.is_changed() {
        return;
    }

    let mut food_position = food_query.iter_mut().next().unwrap();
    move_to(&mut food_position, &snapshot.food, &draw_configuration);

    if snapshot.food_ate {
        food_ate_event_writer.send(FoodAteEvent);
    }
}

fn change_direction(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut current_direction: ResMut<Direction>,
) {
    let direction = keyboard_input_events
        .iter()
        .filter_map(|ki| ki.key_code)
        .filter_map(|kc| match kc {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        })
        .last();

    *current_direction = match direction {
        None => return,
        Some(d) => d,
    };
}

fn game_over(
    mut game_over_event_reader: EventReader<GameOverEvent>,
    mut game_timers: ResMut<GameTimerResource>,
) {
    if game_over_event_reader.iter().count() == 0 {
        return;
    }

    let game_timers = &mut game_timers.0;
    game_timers.pause();
}

fn draw_snake(
    mut commands: Commands,
    bundles: Res<PbrBundles>,
    draw_configuration: Res<DrawConfigurationResource>,
    snapshot: Res<SnakeGameSnapshot>,
) {
    for (i, position) in snapshot.snake.iter().enumerate() {
        let is_head = i == 0;

        if is_head {
            let mut snake_head = bundles.snake_head();

            move_to(&mut snake_head.transform, position, &draw_configuration);
            snake_head.transform.translation.z = SNAKE_Z;

            commands.spawn_bundle(snake_head).insert(HeadSnakeComponent);
        } else {
            spawn_snake_body(&mut commands, &bundles, position, &draw_configuration);
        };
    }
}

fn draw_field(
    mut commands: Commands,
    bundles: Res<PbrBundles>,
    draw_configuration: Res<DrawConfigurationResource>,
    game: Res<SnakeGame>,
    snapshot: Res<SnakeGameSnapshot>,
) {
    let dimension = game.dimension();

    // Create field
    for y in 0..dimension.1 {
        for x in 0..dimension.0 {
            match game.on_walls(&Position::new(x, y)) {
                true => {}
                false => continue,
            };

            let mut wall = bundles.wall();
            move_to(
                &mut wall.transform,
                &Position::new(x, y),
                &draw_configuration,
            );
            wall.transform.translation.z = WALL_Z;

            commands.spawn_bundle(wall);
        }
    }

    // Create initial food
    let food_position = &snapshot.food;
    let mut food = bundles.food();
    move_to(&mut food.transform, food_position, &draw_configuration);
    food.transform.translation.z = FOOD_Z;

    commands.spawn_bundle(food).insert(FoodComponent);
}

fn spawn_snake_body(
    commands: &mut Commands,
    bundles: &PbrBundles,
    position: &Position,
    draw_configuration: &DrawConfigurationResource,
) {
    let mut sprite = bundles.snake_body();
    move_to(&mut sprite.transform, position, draw_configuration);
    // Force snake be rendered over the walls
    sprite.transform.translation.z = SNAKE_Z;

    commands.spawn_bundle(sprite).insert(BodySnakeComponent);
}

fn move_to(
    transform: &mut Transform,
    to: &Position,
    draw_configuration: &DrawConfigurationResource,
) {
    transform.translation.x = to.x as f32 * draw_configuration.cell_size
        + draw_configuration.origin.0
        + draw_configuration.half_cell;
    transform.translation.y = to.y as f32 * draw_configuration.cell_size
        + draw_configuration.origin.1
        + draw_configuration.half_cell;
}

pub struct PbrBundles {
    wall: SpriteBundle,
    snake_head: SpriteBundle,
    snake_body: SpriteBundle,
    food: SpriteBundle,
}
impl PbrBundles {
    pub fn wall(&self) -> SpriteBundle {
        self.wall.clone()
    }
    pub fn snake_body(&self) -> SpriteBundle {
        self.snake_body.clone()
    }
    pub fn snake_head(&self) -> SpriteBundle {
        self.snake_head.clone()
    }
    pub fn food(&self) -> SpriteBundle {
        self.food.clone()
    }
}
impl FromWorld for PbrBundles {
    fn from_world(world: &mut World) -> Self {
        let draw_configuration = world.resource::<DrawConfigurationResource>().clone();

        let mut asset_server = world.resource_mut::<AssetServer>();

        let wall = load_sprite(&mut asset_server, "wall.png", &draw_configuration);
        let snake_body = load_sprite(&mut asset_server, "snake_body.png", &draw_configuration);
        let snake_head = load_sprite(&mut asset_server, "snake_head.png", &draw_configuration);
        let food = load_sprite(&mut asset_server, "food.png", &draw_configuration);

        PbrBundles {
            wall,
            snake_body,
            snake_head,
            food,
        }
    }
}

fn load_sprite(
    asset_server: &mut AssetServer,
    s: &'static str,
    draw_configuration: &DrawConfigurationResource,
) -> SpriteBundle {
    let cell_size = draw_configuration.cell_size;
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(cell_size, cell_size)),
            ..default()
        },
        texture: asset_server.load(s),
        ..default()
    }
}

mod resources {
    use bevy::time::Timer;

    #[derive(Clone)]
    pub struct DrawConfigurationResource {
        pub half_cell: f32,
        pub cell_size: f32,
        pub origin: (f32, f32),
    }

    pub struct GameTimerResource(pub Timer);
}

pub mod events {
    use crate::snake::SnakeError;

    pub struct GameTick;

    pub struct GameOverEvent {
        pub error: SnakeError,
    }
    pub struct FoodAteEvent;
}

mod components {
    use bevy::prelude::Component;

    #[derive(Component)]
    pub struct HeadSnakeComponent;

    #[derive(Component)]
    pub struct BodySnakeComponent;

    #[derive(Component)]
    pub struct FoodComponent;
}
