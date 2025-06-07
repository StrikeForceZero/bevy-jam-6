// copied from https://github.com/Weasy666/bevy_svg/blob/52b79f133e49d1ec50e6ebf0d2be30deaf2a30af/examples/common/lib.rs
// original license: MIT or Apache 2.0

use bevy::color::palettes::basic::GREEN;
use bevy::color::palettes::css::GOLD;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::text::TextSpanAccess;
use bevy_auto_plugin::auto_plugin::*;
use smart_default::SmartDefault;

use super::DebugUiEnabled;

#[derive(Component)]
#[auto_name]
struct FpsText;

#[derive(Component)]
#[auto_name]
struct FpsMinText;

#[derive(Component)]
#[auto_name]
struct FpsMaxText;

#[derive(Component)]
#[auto_name]
struct FrameTimeText;

#[derive(Component)]
#[auto_name]
struct FpsTextRoot;

#[derive(Resource)]
struct FpsValues {
    min: f64,
    max: f64,
}

impl Default for FpsValues {
    fn default() -> Self {
        Self {
            min: 10000.0,
            max: 0.0,
        }
    }
}

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Copy, Clone, SmartDefault, PartialEq, Reflect)]
#[reflect(Resource)]
struct FpsTextScale(#[default(0.5)] f32);

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Copy, Clone, SmartDefault, PartialEq, Reflect)]
#[reflect(Resource)]
struct FpsTextBgColor(#[default(Color::BLACK)] Color);

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Copy, Clone, SmartDefault, PartialEq, Reflect)]
#[reflect(Resource)]
enum FpsTextAnchor {
    Left,
    #[default]
    Right,
}

impl FpsTextAnchor {
    fn text_layout(self) -> TextLayout {
        match self {
            Self::Left => TextLayout::new_with_justify(JustifyText::Left),
            Self::Right => TextLayout::new_with_justify(JustifyText::Right),
        }
    }
    fn right(self, px: f32) -> Val {
        match self {
            Self::Left => Val::Auto,
            Self::Right => Val::Px(px),
        }
    }

    fn left(self, px: f32) -> Val {
        match self {
            Self::Left => Val::Px(px),
            Self::Right => Val::Auto,
        }
    }
}

fn setup_fps_counter(
    mut commands: Commands,
    fps_text_scale: Res<FpsTextScale>,
    fps_text_bg_color: Res<FpsTextBgColor>,
    anchor: Res<FpsTextAnchor>,
    ui_enabled: Res<DebugUiEnabled>,
    fps_text_root: Option<Single<Entity, With<FpsTextRoot>>>,
) {
    if let Some(fps_text_root) = fps_text_root {
        commands.entity(*fps_text_root).despawn();
    }
    if !ui_enabled.0 {
        return;
    }
    let default_text_scale =
        TextFont::from_font_size(TextFont::default().font_size * fps_text_scale.0);
    commands
        .spawn((
            Text::default(),
            TextColor::WHITE,
            TextFont::from_font_size(20.0 * fps_text_scale.0),
            BackgroundColor(fps_text_bg_color.0),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: anchor.left(15.0),
                right: anchor.right(15.0),
                ..default()
            },
            anchor.text_layout(),
            FpsTextRoot,
        ))
        .with_children(|commands| {
            commands
                .spawn((
                    Text::default(),
                    anchor.text_layout(),
                    Node {
                        margin: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                ))
                .with_children(|commands| {
                    commands.spawn((
                        TextSpan::new("FPS: "),
                        TextFont::from_font_size(30.0 * fps_text_scale.0),
                    ));
                    commands.spawn((
                        TextSpan::default(),
                        TextFont::from_font_size(30.0 * fps_text_scale.0),
                        TextColor::from(GOLD),
                        FpsText,
                    ));
                    commands.spawn((TextSpan::new("\n(min: "), default_text_scale.clone()));
                    commands.spawn((
                        TextSpan::default(),
                        TextColor::from(GOLD),
                        default_text_scale.clone(),
                        FpsMinText,
                    ));
                    commands.spawn((TextSpan::new(" - max: "), default_text_scale.clone()));
                    commands.spawn((
                        TextSpan::default(),
                        TextColor::from(GOLD),
                        default_text_scale.clone(),
                        FpsMaxText,
                    ));
                    commands.spawn((TextSpan::new(")"), default_text_scale.clone()));
                    commands.spawn((
                        TextSpan::new("\nms/frame: "),
                        TextFont::from_font_size(30.0 * fps_text_scale.0),
                    ));
                    commands.spawn((
                        TextSpan::default(),
                        TextFont::from_font_size(30.0 * fps_text_scale.0),
                        TextColor::from(GREEN),
                        FrameTimeText,
                    ));
                });
        });
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_values: Local<FpsValues>,
    mut query: ParamSet<(
        Query<&mut TextSpan, With<FpsText>>,
        Query<&mut TextSpan, With<FpsMinText>>,
        Query<&mut TextSpan, With<FpsMaxText>>,
        Query<&mut TextSpan, With<FrameTimeText>>,
        Query<&mut Visibility, With<FpsTextRoot>>,
    )>,
) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_smoothed) = fps.smoothed() {
            if let Ok(mut text) = query.p0().single_mut() {
                *text.write_span() = format!("{fps_smoothed:.2}");
            }
            fps_values.min = fps_values.min.min(fps_smoothed);
            if let Ok(mut text) = query.p1().single_mut() {
                *text.write_span() = format!("{:.2}", fps_values.min);
            }
            fps_values.max = fps_values.max.max(fps_smoothed);
            if let Ok(mut text) = query.p2().single_mut() {
                *text.write_span() = format!("{:.2}", fps_values.max);
            }
        }
    }
    if let Some(frame_time) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(frame_time_smoothed) = frame_time.smoothed() {
            if let Ok(mut text) = query.p3().single_mut() {
                *text.write_span() = format!("{frame_time_smoothed:.2}");
            }
        }
    }
}

fn is_changed(
    anchor: Res<FpsTextAnchor>,
    scale: Res<FpsTextScale>,
    color: Res<FpsTextBgColor>,
    ui_enabled: Res<DebugUiEnabled>,
) -> bool {
    anchor.is_changed() || scale.is_changed() || color.is_changed() || ui_enabled.is_changed()
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup_fps_counter)
        .add_systems(
            Update,
            (setup_fps_counter.run_if(is_changed), fps_text_update_system),
        );
}
