use bevy::{app::AppExit, prelude::*};
use bevy_vector_shapes::prelude::*;
const PLAYER_SIZE: f32 = 10f32;

const PLAYER_THRUST: f32 = 5.0;
const PLAYER_TURN_SPEED: f32 = std::f32::consts::PI / 24.0;

#[derive(Component, Debug, Default, PartialEq)]
struct Velocity(Vec2);

#[derive(Component)]
struct Person;

#[derive(Component, Debug, Eq, PartialEq)]
struct Name(String);

#[derive(Component, Default)]
struct Ship {
    angle: f32,
    thrusting: bool,
}

#[derive(Bundle)]
struct PlayerBundle {
    name: Name,
    ship: Ship,
    position: TransformBundle,
    velocity: Velocity,
}

#[derive(Resource)]
struct GreetTimer(Timer);

fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    let mut window = windows.get_single_mut().unwrap();
    window.title = "Asteroids".to_string();

    let width = window.physical_width();
    let height = window.physical_height();
    println!("width: {}", width);
    println!("height: {}", height);

    commands.spawn(Camera2dBundle::default());

    commands.spawn(PlayerBundle {
        name: Name("Player".to_string()),
        position: TransformBundle::default(),
        velocity: Velocity(Vec2::new(5.0, 10.0)),
        ship: Ship::default(),
    });

    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

// fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
//     // update our timer with the time elapsed since the last update
//     // if that caused the timer to finish, we say hello to everyone
//     if timer.0.tick(time.delta()).just_finished() {
//         for name in &query {
//             println!("hello {}!", name.0);
//         }
//     }
// }

fn quit_on_escape(keyboard_input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn handle_input(keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&mut Ship, &mut Velocity)>) {
    for (ref mut ship, mut velocity) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            ship.angle += PLAYER_TURN_SPEED;
        }
        if keyboard_input.pressed(KeyCode::D) {
            ship.angle -= PLAYER_TURN_SPEED;
        }
        if keyboard_input.pressed(KeyCode::W) {
            velocity.0 += Vec2::new(ship.angle.cos(), ship.angle.sin()) * PLAYER_THRUST;
            ship.thrusting = true;
        } else {
            ship.thrusting = false;
        }
    }
}

fn draw_player(query: Query<(&Ship, &GlobalTransform)>, mut painter: ShapePainter) {
    for (player, position) in &mut query.iter() {
        painter.set_translation(position.translation());
        painter.color = Color::WHITE;
        painter.disable_laa = true;
        painter.thickness_type = ThicknessType::Pixels;
        painter.thickness = 2.0;

        let line_length = 15.0;
        painter.line(
            Vec3::ZERO,
            Vec3::new(player.angle.cos(), player.angle.sin(), 0.0) * line_length,
        );

        let p1 = Vec3::new(
            player.angle.cos() * PLAYER_SIZE,
            player.angle.sin() * PLAYER_SIZE,
            0.0,
        );
        let p2 = Vec3::new(
            (player.angle + std::f32::consts::PI / 4.0 + std::f32::consts::PI).cos() * PLAYER_SIZE,
            (player.angle + std::f32::consts::PI / 4.0 + std::f32::consts::PI).sin() * PLAYER_SIZE,
            0.0,
        );
        let p3 = Vec3::new(
            (player.angle - std::f32::consts::PI / 4.0 + std::f32::consts::PI).cos() * PLAYER_SIZE,
            (player.angle - std::f32::consts::PI / 4.0 + std::f32::consts::PI).sin() * PLAYER_SIZE,
            0.0,
        );
        painter.line(p1, p2);
        painter.line(p2, p3);
        painter.line(p3, p1);
        // println!("player position: {:?}", position);
    }
}

fn move_objects(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (ref mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
    }
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_startup_system(setup)
            .add_system(handle_input)
            .add_system(quit_on_escape)
            .add_system(move_objects)
            .add_system(draw_player);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Shape2dPlugin::default())
        .add_plugin(HelloPlugin)
        .run();
}
